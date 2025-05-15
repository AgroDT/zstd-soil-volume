use args::{Args, Command};
use clap::Parser;

mod args;
mod commands;

fn main() {
    let Args { command } = Args::parse();

    let result = match command {
        Command::Encode(args) => commands::encode::run(args),
    };

    if let Err(err) = result {
        let err = console::style(err).red();
        eprintln!("{err}");
        std::process::exit(1);
    }
}
