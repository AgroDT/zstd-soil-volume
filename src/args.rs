use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[clap(version, about)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Create a new ZSTD volume from a stack of BMP images
    Encode(commands::encode::Args),
}
