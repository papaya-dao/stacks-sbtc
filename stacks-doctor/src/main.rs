use clap::Parser;
use commands::analyze_logs;

use crate::cli::Args;

mod cli;
mod commands;

fn main() {
    let args = Args::parse();

    match &args.cmd {
        cli::Commands::Analyze => analyze_logs(args.log_file),
        cli::Commands::Burns(burns_args) => commands::burns(args.clone(), burns_args.clone()),
        cli::Commands::Env => commands::show_env(),
    }
    .unwrap();
}
