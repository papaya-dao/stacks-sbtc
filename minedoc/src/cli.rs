use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    Analyze,
}

/// Tool for debugging running Stacks nodes
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL to the node RPC API
    #[arg(short, long, env)]
    pub rpc_url: String,

    /// Path to the node log file
    #[arg(short, long, env)]
    pub log_file: PathBuf,

    /// Path to the node db file
    #[arg(short, long, env)]
    pub db_file: PathBuf,

    #[command(subcommand)]
    pub cmd: Commands,
}
