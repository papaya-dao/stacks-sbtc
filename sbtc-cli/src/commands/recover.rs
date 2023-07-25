use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::sha256::Hash;
use bitcoin::secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
use bitcoin::secp256k1::{Message, Secp256k1};
use clap::Parser;
use std::io::stdout;

#[derive(Parser, Debug, Clone)]
pub struct RecoverArgs {
    /// P2WPKH BTC private key in WIF format
    #[clap(short, long)]
    message_hex: String,

    #[clap(short, long)]
    signature: String,

    #[clap(short, long)]
    recovery_id: i32,
}

pub fn recover(args: &RecoverArgs) -> anyhow::Result<()> {
    let recovery_id = RecoveryId::from_i32(args.recovery_id)?;
    let msg = Message::from_hashed_data::<Hash>(
        array_bytes::hex2bytes(&args.message_hex)
            .expect("hex2bytes")
            .as_slice(),
    );
    let signature = RecoverableSignature::from_compact(
        array_bytes::hex2bytes(&args.signature)
            .expect("hex2bytes")
            .as_slice(),
        recovery_id,
    )?;

    let s = Secp256k1::new();
    let public_key = s.recover_ecdsa(&msg, &signature)?;

    serde_json::to_writer_pretty(stdout(), &public_key.serialize().to_hex())?;

    Ok(())
}
