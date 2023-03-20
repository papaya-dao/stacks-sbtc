use clap::Parser;

use crate::cli::Args;

mod cli;
mod commands;

fn main() {
    let args = Args::parse();

    match args.cmd {
        cli::Commands::Analyze(subcmd) => {
            let is_okay = match subcmd {
                cli::AnalyzeCommands::RPC(_subcmd_args) => todo!(),
                cli::AnalyzeCommands::Logs(subcmd_args) => {
                    commands::analyze_logs(subcmd_args.log_file)
                }
                cli::AnalyzeCommands::DB(_subcmd_args) => todo!(),
                cli::AnalyzeCommands::All(_subcmd_args) => todo!(),
            };

            if is_okay {
                println!("No problems detected");
            } else {
                println!("Problems detected");
            }
        }
        cli::Commands::Burns(burns_args) => commands::burns(burns_args),
        cli::Commands::Env => commands::show_env(),
    };
}
