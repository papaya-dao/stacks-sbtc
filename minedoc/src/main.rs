use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    Analyze,
}

/// Tool for debugging running Stacks nodes
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to the node RPC API
    #[arg(short, long, env)]
    rpc_url: String,

    /// Path to the node log file
    #[arg(short, long, env)]
    log_file: PathBuf,

    /// Path to the node db file
    #[arg(short, long, env)]
    db_file: PathBuf,

    #[command(subcommand)]
    cmd: Commands,
}

fn main() {
    let args = Args::parse();

    dbg!(args);
}
