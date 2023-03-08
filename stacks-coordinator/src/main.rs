use clap::Parser;
use frost_signer::logging;
use stacks_coordinator::cli::{Cli, Command};

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
            // Spawn a coordinator with a receiver channel to receive incoming commands to handle
            // Must know how to communicate with the blockchain
            todo!();
        }
        Command::Dkg => {
            // Spawn a frost coordinator and begin a singing round
            // Must know what stacks-signers are participating in the round
            todo!()
        }
    };
}
