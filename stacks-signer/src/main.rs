use clap::Parser;
use frost_signer::logging;
use stacks_signer::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    logging::initiate_tracing_subscriber(if cli.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    })
    .unwrap();

    // Determine what action the caller wishes to perform
    match cli.command {
        Command::Run => {
            // Set up p2p network and listen for incoming messages
            todo!();
        }
        Command::Secp256k1(secp256k1) => {
            secp256k1.generate_private_key().unwrap();
        }
    };
}
