use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct AnalyzeRPCArgs {
    /// URL to the node RPC API
    #[arg(short, long, env = "MINEDOC_RPC_URL")]
    pub rpc_url: String,
}

#[derive(Parser, Debug)]
pub struct AnalyzeLogsArgs {
    /// Path to the node log file
    #[arg(short, long, env = "MINEDOC_LOG_FILE")]
    pub log_file: PathBuf,
}

#[derive(Parser, Debug)]
pub struct AnalyzeDBArgs {
    /// Path to the node db file
    #[arg(short, long, env = "MINEDOC_DB_FILE")]
    pub db_file: PathBuf,
}

// This can't combine previous Args structs as it's limited by the clap parser
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct AnalyzeAllArgs {
    /// URL to the node RPC API
    #[arg(short, long, env = "MINEDOC_RPC_URL")]
    pub rpc_url: String,

    /// Path to the node log file
    #[arg(short, long, env = "MINEDOC_LOG_FILE")]
    pub log_file: PathBuf,

    /// Path to the node db file
    #[arg(short, long, env = "MINEDOC_DB_FILE")]
    pub db_file: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum AnalyzeCommands {
    /// Use RPC API
    RPC(AnalyzeRPCArgs),
    /// Use logs
    Logs(AnalyzeLogsArgs),
    /// Use database
    DB(AnalyzeDBArgs),
    /// Use all data sources
    All(AnalyzeAllArgs),
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze miner
    #[command(subcommand)]
    Analyze(AnalyzeCommands),
    /// Print related environment variables that are set
    Env,
}

/// Tool for debugging running Stacks nodes
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Commands,
}
