use clap::Parser;

///Command line interface for stacks coordinator
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    /// Subcommand to perform
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    // Listen for incoming peg in and peg out requests.
    Run,
    // Run distributed key generation round
    Dkg,
}
