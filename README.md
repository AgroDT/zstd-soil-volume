# zstd-soil-volume

A fast, memory-efficient CLI utility for encoding stacks of BMP soil tomography
slices into a single compressed `.raw.zst` file — ready to be streamed and
visualized in the browser with
[three-zstd-volume-loader](https://github.com/AgroDT/three-zstd-volume-loader)
and [three-soil-volume-shader](https://github.com/AgroDT/three-soil-volume-shader).

## Features

- Accepts a directory of BMP slices as input
- Compresses voxel volumes using Zstandard, with a user-defined compression level
- Streams image loading, compression, and file writing for minimal RAM usage
- Written in Rust, so naturally it's *blazingly fast™*
- Outputs a single `.raw.zst` file for browser-friendly visualization
- Packaged as a single static executable — works anywhere, no dependencies,
  no installation hassle

## Installation

Download a prebuilt binary for your platform from the
[Releases](https://github.com/AgroDT/zstd-soil-volume/releases/latest),
or build from source with:

```sh
cargo install --git https://github.com/AgroDT/zstd-soil-volume
```

## Usage

Currently, `zstd-soil-volume` provides the `encode` subcommand:

```sh
zstd-soil-volume encode [OPTIONS] -o <PATH> <BMP_DIR>
```

Where:

- `<BMP_DIR>` — a directory containing a sequence of BMP images
  (e.g., sample__rec0000_bin_0001.bmp..sample__rec0000_bin_0376.bmp)
- `-o <PATH>` — the output path for the compressed `.raw.zst` volume

Compression is performed on-the-fly without preloading the entire dataset into
memory.

### Example

```sh
zstd-soil-volume encode -l 19 ./slices/ -o ./volume.raw.zst
```

This command encodes the BMP stack in `./slices/` using compression level 19
into a single compressed file `volume.raw.zst`.

## CLI Options

To view help messages, run:

```text
$ zstd-soil-volume help
CLI tools to work with three.js ZSTD soil volumes

Usage: zstd-soil-volume.exe <COMMAND>

Commands:
  encode  Create a new ZSTD volume from a stack of BMP images
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```text
$ zstd-soil-volume help encode
Create a new ZSTD volume from a stack of BMP images

Usage: zstd-soil-volume.exe encode [OPTIONS] --output <PATH> <BMP_DIR>

Arguments:
  <BMP_DIR>  Directory with BMP files

Options:
  -o, --output <PATH>           Path to output file
  -f, --force                   Overwrite existing files
  -l, --zstd-level <LEVEL>      ZSTD compression level (1-22) [default: 3]
  -t, --zstd-threads <THREADS>  ZSTD thread count, 0 disables multithreading [default: 0]
  -h, --help                    Print help
```

## Compatibility

The output `.raw.zst` volume is designed to be used with:

- [three-zstd-volume-loader](https://github.com/AgroDT/three-zstd-volume-loader)
- [three-soil-volume-shader](https://github.com/AgroDT/three-soil-volume-shader)

## Development

After cloning the repository, enable Git hooks to automatically run pre-commit
checks:

```sh
make init
```

Or set the hook path manually:

```sh
git config core.hooksPath .git-hooks
```

## License

This project is licensed under the MIT License.
