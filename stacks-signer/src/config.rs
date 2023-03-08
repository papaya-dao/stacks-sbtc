use serde::Deserialize;
use std::fs;
use toml;

#[derive(Clone, Deserialize, Default, Debug)]
pub struct Config {
    pub stacks_node_url: String,
    pub total_signers: usize,
    pub total_parties: usize,
    pub minimum_parties: usize,
}

impl Config {
    //Create a config file from a given toml filepath.
    //TODO: get config info from sBTC contracts
    pub fn from_file(path: &str) -> Result<Config, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Invalid path: {}", &e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid toml: {}", e))
    }
}
