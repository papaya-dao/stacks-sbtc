extern crate core;

use clap::Parser;

use crate::commands::broadcast::{broadcast_tx, BroadcastArgs};
use crate::commands::deposit::{build_deposit_tx, DepositArgs};
use crate::commands::generate::{generate, GenerateArgs};
use crate::commands::recover::{recover, RecoverArgs};
use crate::commands::sign::{sign, SignArgs};
use crate::commands::withdraw::{build_withdrawal_tx, WithdrawalArgs};

mod commands;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Command {
    Deposit(DepositArgs),
    // cargo run -p sbtc-cli -- withdraw --wif $WIF --sender-wif $WIF --recipient tb1q0jtfel9tp54dzud28uspe994rv8gajnxc85n8q --amount 42 --dkg-wallet
    // tb1pewpc7x6nnea8clm2vn2d8xvpdwvkhucmfdwmm0p6vk2u5xgmwlzsdx3g6w --fulfillment-fee 1000
    Withdraw(WithdrawalArgs),
    Broadcast(BroadcastArgs),
    GenerateFrom(GenerateArgs),
    // cargo run -p sbtc-cli -- sign --wif $WIF --message-hex 0xdeadbeef
    Sign(SignArgs),
    // cargo run -p sbtc-cli -- recover --message-hex 0xdeadbeef --signature 0xfe9fcbe92ce55853bb2339c95f126bf46288a8f3e6a98aaf7730ccd73d984f8a1224f68325afca456d35eb40d692102ae24ac04fb0c7d315194624f2c34ccc5c --recovery-id 1
    Recover(RecoverArgs),
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    match args.command {
        Command::Deposit(args) => build_deposit_tx(&args),
        Command::Withdraw(args) => build_withdrawal_tx(&args),
        Command::Broadcast(args) => broadcast_tx(&args),
        Command::GenerateFrom(args) => generate(&args),
        Command::Sign(args) => sign(&args),
        Command::Recover(args) => recover(&args),
    }
}
