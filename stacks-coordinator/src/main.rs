use stacks_coordinator::cli;

fn main() {
    let args = cli::Args::parse();

    match args.command {
        cli::Command::Run => {
            println!("Running coordinator");
        }
        cli::Command::Dkg => {
            println!("Running DKG");
        }
    };
}
