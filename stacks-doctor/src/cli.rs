use core::fmt::{self, Formatter};
use std::{
    ascii::AsciiExt,
    fmt::{Debug, Display},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct AnalyzeRPCArgs {
    /// URL to the node RPC API
    #[arg(short, long, env = "DOCTOR_RPC_URL")]
    pub rpc_url: String,
}

#[derive(Parser, Debug)]
pub struct AnalyzeLogsArgs {
    /// Path to the node log file
    #[arg(short, long, env = "DOCTOR_LOG_FILE")]
    pub log_file: PathBuf,
}

#[derive(Parser, Debug)]
pub struct AnalyzeDBArgs {
    /// Path to the node db file
    #[arg(short, long, env = "DOCTOR_DB_FILE")]
    pub db_file: PathBuf,
}

// This can't combine previous Args structs as it's limited by the clap parser
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct AnalyzeAllArgs {
    /// URL to the node RPC API
    #[arg(short, long, env = "DOCTOR_RPC_URL")]
    pub rpc_url: String,

    /// Path to the node log file
    #[arg(short, long, env = "DOCTOR_LOG_FILE")]
    pub log_file: PathBuf,

    /// Path to the node db file
    #[arg(short, long, env = "DOCTOR_DB_FILE")]
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

#[derive(Debug, Clone)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => Err(format!("Could not parse Network: {}", s)),
        }
    }
}

#[derive(Parser, Debug)]
pub struct BurnsArgs {
    /// How many recent recipients to take into account
    #[arg(short, long, default_value_t = 250)]
    pub recipients: u64,

    /// Which network to analyze
    #[arg(short, long, default_value_t = Network::Mainnet)]
    pub network: Network,

    // How many recent blocks to take into account
    #[arg(short, long, default_value_t = 1000)]
    pub blocks: u64,

    /// Path to the node db file
    #[arg(short, long, env = "DOCTOR_DB_DIR")]
    pub db_dir: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze miner
    #[command(subcommand)]
    Analyze(AnalyzeCommands),
    /// Print information about burn amount for recent reward recipiets
    Burns(BurnsArgs),
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
