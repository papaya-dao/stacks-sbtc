use clap::Parser;

use crate::cli::{Args, Commands};

mod cli;
mod commands;

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Analyze => commands::analyze(args),
    }
}
