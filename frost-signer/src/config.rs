use clap::Parser;
use hashbrown::HashMap;
use p256k1::ecdsa;
use serde::Deserialize;
use std::fs;
use toml;

use crate::util::parse_public_key;

#[derive(Default)]
pub struct SignerKeys {
    pub signers: HashMap<u32, ecdsa::PublicKey>,
    pub key_ids: HashMap<u32, ecdsa::PublicKey>,
}

#[derive(Clone, Deserialize, Default, Debug)]
struct SignerDataConfig {
    pub public_key: String,
    pub key_ids: Vec<u32>,
}

#[derive(Clone, Deserialize, Default, Debug)]
pub struct Config {
    pub http_relay_url: String,
    pub total_signers: u32,
    pub total_keys: usize,
    pub keys_threshold: usize,
    pub frost_state_file: String,
    pub network_private_key: String,
    signers: Vec<SignerDataConfig>,
    coordinator_public_key: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Config file path
    #[arg(short, long)]
    pub config: String,

    /// Start a signing round
    #[arg(short, long)]
    pub start: bool,

    /// ID associated with signer
    #[arg(short, long)]
    pub id: u32,
}

impl Config {
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Config, Error> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn signers(&self) -> Result<SignerKeys, Error> {
        let mut signer_keys = SignerKeys::default();
        for (i, s) in self.signers.iter().enumerate() {
            let signer_public_key = parse_public_key(&s.public_key).map_err(|_| {
                Error::InvalidPublicKey(format!(
                    "Failed to parse signers from config. {}",
                    s.public_key
                ))
            })?;
            s.key_ids.iter().for_each(|key_id| {
                signer_keys.key_ids.insert(*key_id, signer_public_key);
            });
            // We start our signer ids from 1 not 0, hence i + 1
            let k = (i + 1).try_into().unwrap();
            signer_keys.signers.insert(k, signer_public_key);
        }
        Ok(signer_keys)
    }

    pub fn coordinator_public_key(&self) -> Result<ecdsa::PublicKey, Error> {
        parse_public_key(&self.coordinator_public_key).map_err(|_| {
            Error::InvalidPublicKey(format!(
                "Failed to parse coordinator_public_key from config. {}",
                self.coordinator_public_key
            ))
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    #[error("Invalid Public Key: {0}")]
    InvalidPublicKey(String),
}
