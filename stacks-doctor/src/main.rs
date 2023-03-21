use clap::Parser;

use crate::cli::Args;

mod cli;
mod commands;

fn main() {
    let args = Args::parse();

    match &args.cmd {
        cli::Commands::Analyze => todo!(),
        cli::Commands::Burns(burns_args) => commands::burns(args.clone(), burns_args.clone()),
        cli::Commands::Env => commands::show_env(),
    };
}
