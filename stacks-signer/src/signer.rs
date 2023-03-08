use crate::cli::Cli;
use serde::Deserialize;
use std::fs;
use toml;

#[derive(Clone, Deserialize, Default, Debug)]
pub struct Config {
    pub common: Common,
    pub signer: Signer,
}

#[derive(Clone, Deserialize, Default, Debug)]
pub struct Common {
    pub stacks_node_url: String,
    pub total_signers: usize,
    pub total_parties: usize,
    pub minimum_parties: usize,
}

// on-disk format for stacks save data
#[derive(Clone, Deserialize, Default, Debug)]
pub struct Signer {
    pub id: u32,
    pub state_file: String,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Invalid path: {}", &e))?;
        Ok(toml::from_str(&content).map_err(|e| format!("Invalid toml: {}", e))?)
    }

    pub fn merge(&mut self, cli: &Cli) {
        if let Some(signer_id) = cli.id {
            self.signer.id = signer_id;
        }
    }
}
