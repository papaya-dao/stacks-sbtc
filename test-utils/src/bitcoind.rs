use bitcoin::secp256k1::{self, All, Message, Secp256k1, SecretKey};
use bitcoin::util::sighash::SighashCache;
use bitcoin::{
    Address, EcdsaSighashType, Network, PrivateKey, PublicKey, Transaction, TxOut, Txid,
};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use ureq::serde::Serialize;
use ureq::serde_json::Value;

use crate::process::Process;
use crate::util::find_port;

const BITCOIND_URL: &str = "http://abcd:abcd@localhost";

pub struct BitcoinProcess {
    datadir: PathBuf,
    process: Process,
}

impl BitcoinProcess {
    pub fn new() -> Self {
        let mut datadir: PathBuf = PathBuf::from_str("/tmp/").unwrap();
        let tempfile: String = "bitcoind_test_"
            .chars()
            .chain(
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(16)
                    .map(char::from),
            )
            .collect();

        datadir = datadir.join(tempfile);
        create_dir(&datadir).unwrap();

        let port = find_port().unwrap();

        let bitcoind_child = Command::new("bitcoind")
            .arg("-regtest")
            .arg("-rpcuser=abcd")
            .arg("-rpcpassword=abcd")
            .arg(format!("-rpcport={}", port))
            .arg(format!("-datadir={}", datadir.to_str().unwrap()))
            .stdout(Stdio::null())
            .spawn()
            .expect("bitcoind failed to start");

        let process = Process::new(BITCOIND_URL, port, "bitcoind".to_string(), bitcoind_child);

        Self { process, datadir }
    }

    pub fn rpc(&self, method: &str, params: impl Serialize) -> Value {
        self.process.rpc(method, params)
    }
}

impl Drop for BitcoinProcess {
    fn drop(&mut self) {
        remove_dir_all(&self.datadir).unwrap();
    }
}

pub fn generate_wallet() -> (SecretKey, PrivateKey, PublicKey, Address) {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let private_key = PrivateKey::new(secret_key, Network::Regtest);
    let public_key = PublicKey::from_private_key(&secp, &private_key);
    let address = Address::p2wpkh(&public_key, Network::Regtest).unwrap();

    (secret_key, private_key, public_key, address)
}

pub fn mine_and_get_coinbase_txid(btcd: &Process, addr: &Address) -> (Txid, String) {
    let block_id = btcd
        .rpc("generatetoaddress", (100, addr.to_string()))
        .as_array()
        .unwrap()[0]
        .as_str()
        .unwrap()
        .to_string();

    let block = btcd.rpc("getblock", (block_id, 1));
    let blockhash = block.get("hash").unwrap().as_str().unwrap().to_string();

    (
        Txid::from_str(block.get("tx").unwrap().get(0).unwrap().as_str().unwrap()).unwrap(),
        blockhash,
    )
}

pub fn sign_transaction(
    addr: &Address,
    secret_key: &SecretKey,
    public_key: &PublicKey,
    prev_output: &TxOut,
    tx: &mut Transaction,
    secp: &Secp256k1<All>,
) {
    let tx_sighash_pubkey_script = addr.script_pubkey().p2wpkh_script_code().unwrap();
    let mut sighash_cache_peg_in = SighashCache::new(&*tx);

    let tx_sighash = sighash_cache_peg_in
        .segwit_signature_hash(
            0,
            &tx_sighash_pubkey_script,
            prev_output.value,
            EcdsaSighashType::All,
        )
        .unwrap();

    let msg = Message::from_slice(&tx_sighash).unwrap();
    let sig = secp.sign_ecdsa_low_r(&msg, secret_key);
    let secp_public_key_source = secp256k1::PublicKey::from_secret_key(secp, secret_key);

    secp.verify_ecdsa(&msg, &sig, &secp_public_key_source)
        .unwrap();

    tx.input[0]
        .witness
        .push_bitcoin_signature(&sig.serialize_der(), EcdsaSighashType::All);
    tx.input[0]
        .witness
        .push(bitcoin::psbt::serialize::Serialize::serialize(public_key));
}
