use crate::commands::deposit::DepositArgs;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::sha256::Hash;
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::PrivateKey;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::stdout;

#[derive(Parser, Debug, Clone)]
pub struct SignArgs {
    /// P2WPKH BTC private key in WIF format
    #[clap(short, long)]
    wif: String,

    /// hex-encoded message to sign
    #[clap(short, long)]
    message_hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RecoverableSignature {
    pub recovery_id: i32,
    pub signature_hex: String,
}

pub fn sign(args: &SignArgs) -> anyhow::Result<()> {
    let private_key = PrivateKey::from_wif(&args.wif)?;
    let msg = Message::from_hashed_data::<Hash>(
        array_bytes::hex2bytes(&args.message_hex)
            .expect("hex2bytes")
            .as_slice(),
    );

    let mut s = Secp256k1::new();
    let sig = s.sign_ecdsa_recoverable(&msg, &private_key.inner);
    let serialized_sig = sig.serialize_compact();

    serde_json::to_writer_pretty(
        stdout(),
        &RecoverableSignature {
            recovery_id: serialized_sig.0.to_i32(),
            signature_hex: serialized_sig.1.to_hex(),
        },
    )
    .unwrap();

    Ok(())
}
