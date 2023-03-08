use crate::secp256k1::Secp256k1;
use clap::{Parser, Subcommand};

///Command line interface for stacks signer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    /// Associated signer id
    #[arg(short, long)]
    pub id: u32,

    /// Subcommand action to take
    #[clap(subcommand)]
    pub command: Command,
}

/// Possible actions that stacks signer can perform
#[derive(Subcommand)]
pub enum Command {
    /// Join the p2p network as specified in the config file
    Run,
    /// Generate Secp256k1 Private Key
    Secp256k1(Secp256k1),
}
