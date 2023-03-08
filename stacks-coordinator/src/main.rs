use clap::Parser;
use frost_signer::logging;
use stacks_coordinator::cli::{Cli, Command};
use stacks_coordinator::config::Config;

use frost_signer::net::{HttpNet, HttpNetListen};

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    logging::initiate_tracing_subscriber(if cli.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    })
    .unwrap();

    let config = Config::from_file("conf/coordinator.toml").unwrap();

    // Create the relay to talk with the signers. This will need to be passed to the coordinator
    let net: HttpNet = HttpNet::new(config.signer_relay_url);
    let _net_listen: HttpNetListen = HttpNetListen::new(net, vec![]);

    // Determine what action the caller wishes to perform
    match cli.command {
        Command::Run => {
            // get Ops from stacks node RPC
            // Has to handle peg out signing
            todo!();
        }
        Command::Dkg => {
            //Start a signing round
            todo!()
        }
    };
}
