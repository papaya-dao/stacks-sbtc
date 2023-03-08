use clap::Parser;
use frost_signer::logging;
use stacks_signer::cli::{Cli, Command};
use stacks_signer::signer::Signer;
use tracing::info;
use tracing::warn;

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
            let mut signer = Signer::from_file("conf/stacker.toml").unwrap();
            let cli = Cli::parse();
            if let Some(id) = cli.id {
                signer.id = id;
            }
            info!("{} signer id #{}", stacks_signer::version(), signer.id); // sign-on message
            if let Err(e) = signer.create_p2p_sync() {
                warn!("An error occurred in the P2P Network: {}", e);
            }
        }
        Command::Secp256k1(secp256k1) => {
            secp256k1.generate_private_key().unwrap();
        }
    };
}
