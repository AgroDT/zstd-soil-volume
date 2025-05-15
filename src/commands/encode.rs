use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use byteorder::{LittleEndian, WriteBytesExt};
use clio::ClioPath;
use console::Emoji;
use image::GenericImageView;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};

static METADATA_FRAME_MAGIC: u32 = 0x184D2A50;

static FROWNING_FACE: Emoji<'_, '_> = Emoji("☹️  ", ":-/");
static PAGE_WITH_CURL: Emoji<'_, '_> = Emoji("📃  ", "");
static CHECK_MARK_BUTTON: Emoji<'_, '_> = Emoji("✅  ", "");

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Directory with BMP files
    #[clap(value_parser = clap::value_parser!(ClioPath).exists().is_dir())]
    pub(crate) bmp_dir: ClioPath,

    /// Path to output file
    #[clap(short, long, value_name = "PATH")]
    output: PathBuf,

    /// Overwrite existing files
    #[clap(short, long)]
    force: bool,

    /// ZSTD compression level (1-22)
    #[clap(
        short = 'l',
        long,
        default_value = "3",
        value_parser = clap::value_parser!(i32).range(1..23),
        value_name = "LEVEL",
    )]
    pub(crate) zstd_level: i32,

    /// ZSTD compression thread count, 0 disables multithreading
    #[clap(
        short = 't',
        long,
        default_value = "0",
        value_parser,
        value_name = "THREADS"
    )]
    pub(crate) zstd_threads: u32,
}

pub(crate) fn run(args: Args) -> anyhow::Result<()> {
    if !args.force && args.output.is_file() {
        anyhow::bail!(
            r#"Output "{}" already exists, run with `--force` to overwrite"#,
            args.output.to_string_lossy()
        );
    }

    let start = Instant::now();

    let (paths, x_size, y_size) = match BmpPaths::glob(args.bmp_dir.as_ref())? {
        BmpPaths::Empty => return Ok(()),
        BmpPaths::Found {
            paths,
            x_size,
            y_size,
        } => (paths, x_size, y_size),
    };

    let z_size = u64::try_from(paths.len())?;
    let src_size = x_size as u64 * y_size as u64 * z_size;

    let mut writer = std::fs::File::create(&args.output)?;

    let metadata_json =
        format!(r#"{{"xSize":{x_size},"ySize":{y_size},"zSize":{z_size},"type":"uint8"}}"#);
    let metadata = metadata_json.as_bytes();
    writer.write_u32::<LittleEndian>(METADATA_FRAME_MAGIC)?;
    writer.write_u32::<LittleEndian>(u32::try_from(metadata.len())?)?;
    writer.write_all(metadata)?;

    let mut writer = zstd::stream::Encoder::new(writer, args.zstd_level)?;
    writer.multithread(args.zstd_threads)?;
    writer.set_pledged_src_size(Some(src_size))?;
    writer.long_distance_matching(true)?;
    let mut writer = writer.auto_finish();

    let x_size = x_size as usize;
    let y_size = y_size as usize;
    let mut buffer = vec![0u8; x_size * y_size];

    let pb = ProgressBar::new(z_size);
    pb.enable_steady_tick(Duration::from_millis(500));
    let fmt_len = z_size.checked_ilog10().unwrap_or(2) + 1;
    pb.set_style(ProgressStyle::with_template(
        &format!("{{spinner:.green}}  Processing {{pos:>{fmt_len}}}/{{len}} {{wide_msg}} ({{elapsed}}, ETA {{eta}})"),
    )?);
    for path in &paths {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        pb.set_message(filename);
        let img = image::open(path)?.to_luma8();
        let data = img.as_raw();

        // Convert to fortran ordering
        for x in 0..x_size {
            for y in 0..y_size {
                buffer[x * y_size + y] = data[y * x_size + x];
            }
        }

        writer.write_all(&buffer)?;
        pb.inc(1);
    }
    writer.flush()?;
    pb.finish_and_clear();

    println!(
        r#"{CHECK_MARK_BUTTON}Finished writing {x_size}×{y_size}×{z_size} voxels to "{}" in {}"#,
        args.output.to_string_lossy(),
        HumanDuration(start.elapsed()),
    );

    Ok(())
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum BmpPaths {
    Empty,
    Found {
        paths: Vec<PathBuf>,
        x_size: u32,
        y_size: u32,
    },
}

impl BmpPaths {
    fn glob<'a, P: Into<&'a Path>>(dir: P) -> anyhow::Result<Self> {
        let dir: &Path = dir.into();
        let dir_path = dir.to_string_lossy();

        let pb = ProgressBar::new_spinner()
            .with_style(ProgressStyle::with_template("{spinner:.green}  {msg}")?);
        pb.set_message(format!(r#"Searching for BMP files in "{dir_path}""#));
        pb.tick();

        let pattern = dir.join("*.bmp");
        let options = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut paths = glob::glob_with(&pattern.to_string_lossy(), options)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(anyhow::Error::from)?;

        if paths.is_empty() {
            pb.finish_with_message(format!(
                r#"{FROWNING_FACE}No BMP files found in "{dir_path}""#
            ));
            return Ok(Self::Empty);
        }

        let (x_size, y_size) = image::open(paths.first().unwrap())?.dimensions();

        for path in &paths {
            let (img_x_size, img_y_size) = image::open(path)?.dimensions();
            if img_x_size != x_size || img_y_size != y_size {
                anyhow::bail!(
                    r#"BMP images have different dimensions: first was {x_size}×{y_size}, but "{}" has {img_x_size}×{img_y_size}"#,
                    path.to_string_lossy()
                );
            }
            pb.tick();
        }

        paths.sort();

        pb.finish_and_clear();
        println!("{PAGE_WITH_CURL}Found {} BMP file(s)", paths.len());

        Ok(Self::Found {
            paths,
            x_size,
            y_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::{Path, PathBuf},
    };

    use clio::ClioPath;

    use super::{Args, BmpPaths, METADATA_FRAME_MAGIC, run};

    #[test]
    fn test_bmp_invalid_bmps() {
        let paths = BmpPaths::glob(Path::new("tests/data/invalid-bmps"));
        assert!(paths.is_err());
    }

    #[test]
    fn test_bmp_different_size() {
        let paths = BmpPaths::glob(Path::new("tests/data/different-size-bmps"));
        assert!(paths.is_err());
    }

    #[test]
    fn test_bmp_paths_empty() {
        let paths = BmpPaths::glob(Path::new("tests"));
        assert!(paths.is_ok());
        assert_eq!(paths.unwrap(), BmpPaths::Empty);
    }

    #[test]
    fn test_bmp_paths_ok() {
        let paths = BmpPaths::glob(Path::new("tests/data/valid-bmps"));
        assert!(paths.is_ok());
        if let BmpPaths::Found {
            paths,
            x_size,
            y_size,
        } = paths.unwrap()
        {
            assert_eq!(paths.len(), 2);
            assert_eq!(paths[0], PathBuf::from("tests/data/valid-bmps/1.bmp"));
            assert_eq!(paths[1], PathBuf::from("tests/data/valid-bmps/2.bmp"));
            assert_eq!(x_size, 2);
            assert_eq!(y_size, 2);
        } else {
            panic!("Expected BMPs to be found")
        }
    }

    #[test]
    fn test_run_existing() {
        let res = run(Args {
            bmp_dir: ClioPath::local("tests/data/valid-bmps".into()),
            output: PathBuf::from("tests/data/volume.raw.zst"),
            force: false,
            zstd_level: 22,
            zstd_threads: 0,
        });

        assert!(res.is_err());
    }

    #[test]
    fn test_run_ok() {
        let tempdir = tempfile::tempdir().unwrap();
        let output = tempdir.path().join("volume.raw.zst");
        let res = run(Args {
            bmp_dir: ClioPath::local("tests/data/valid-bmps".into()),
            output: output.clone(),
            force: false,
            zstd_level: 22,
            zstd_threads: 0,
        });

        assert!(res.is_ok());

        let data = zstd::decode_all(File::open(&output).unwrap()).unwrap();
        let expected = zstd::decode_all(File::open("tests/data/volume.raw.zst").unwrap()).unwrap();
        assert_eq!(data, expected);

        let mut header = METADATA_FRAME_MAGIC.to_le_bytes().to_vec();
        let metadata = br#"{"xSize":2,"ySize":2,"zSize":2,"type":"uint8"}"#;
        header.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        header.extend_from_slice(metadata);
        let raw_data = fs::read(&output).unwrap();
        assert_eq!(raw_data[..header.len()], header);
    }
}
