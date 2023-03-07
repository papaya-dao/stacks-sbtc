use frost_coordinator::create_coordinator;
use stacks_coordinator::cli;

use stacks_coordinator::frost_coordinator::FrostCoordinator;

fn main() {
    let args = cli::Args::parse();

    match args.command {
        cli::Command::Run => {
            println!("Running coordinator");
        }
        cli::Command::Dkg => {
            println!("Running DKG");

            let mut coordinator = create_coordinator();
            coordinator.run_dkg_round();
        }
    };
}
