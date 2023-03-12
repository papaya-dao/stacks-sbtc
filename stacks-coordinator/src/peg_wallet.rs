use bitcoin::hashes::Hash;
use serde::Serialize;

use crate::bitcoin_node;
use crate::bitcoin_node::BitcoinTransaction;
use crate::error::Result;
use crate::stacks_node;
use crate::stacks_node::{PegInOp, PegOutRequestOp};
use crate::stacks_transaction::StacksTransaction;

pub trait StacksWallet {
    fn mint(&mut self, op: &stacks_node::PegInOp) -> Result<StacksTransaction>;
    fn burn(&mut self, op: &stacks_node::PegOutRequestOp) -> Result<StacksTransaction>;
    fn set_wallet_address(&mut self, address: PegWalletAddress) -> Result<StacksTransaction>;
}

pub trait BitcoinWallet {
    fn fulfill_peg_out(
        &self,
        op: &stacks_node::PegOutRequestOp,
    ) -> bitcoin_node::BitcoinTransaction;
}

pub trait PegWallet {
    type StacksWallet: StacksWallet;
    type BitcoinWallet: BitcoinWallet;
    fn stacks_mut(&mut self) -> &mut Self::StacksWallet;
    fn bitcoin_mut(&mut self) -> &mut Self::BitcoinWallet;
}

// TODO: Representation
// Should correspond to a [u8; 32] - perhaps reuse a FROST type?
#[derive(Serialize)]
pub struct PegWalletAddress(pub [u8; 32]);

pub struct WrapPegWallet {}

impl PegWallet for WrapPegWallet {
    type StacksWallet = FileStacksWallet;
    type BitcoinWallet = FileBitcoinWallet;

    fn stacks_mut(&mut self) -> &mut Self::StacksWallet {
        todo!()
    }

    fn bitcoin_mut(&mut self) -> &mut Self::BitcoinWallet {
        todo!()
    }
}

pub struct FileStacksWallet {}

impl StacksWallet for FileStacksWallet {
    fn mint(&mut self, _op: &PegInOp) -> Result<StacksTransaction> {
        todo!()
    }

    fn burn(&mut self, _op: &PegOutRequestOp) -> Result<StacksTransaction> {
        todo!()
    }

    fn set_wallet_address(&mut self, _address: PegWalletAddress) -> Result<StacksTransaction> {
        todo!()
    }
}

pub struct FileBitcoinWallet {}

impl BitcoinWallet for FileBitcoinWallet {
    fn fulfill_peg_out(&self, op: &PegOutRequestOp) -> BitcoinTransaction {
        //fn build_peg_out(satoshis: u64, user_address: bitcoin::PublicKey, utxo: OutPoint) -> Transaction
        let bitcoin_txid = bitcoin::Txid::from_slice(op.txid.as_bytes()).unwrap();
        let utxo = bitcoin::OutPoint {
            txid: bitcoin_txid,
            vout: op.vtxindex,
        };
        let peg_out_input = bitcoin::TxIn {
            previous_output: utxo,
            script_sig: Default::default(),
            sequence: Default::default(),
            witness: Default::default(),
        };
        // todo: op.recipient should be a bitcoin address
        let user_address = bitcoin::PublicKey::from_slice(&op.recipient.bytes()).unwrap();
        let p2wpk = bitcoin::Script::new_v0_p2wpkh(&user_address.wpubkey_hash().unwrap());
        let peg_out_output = bitcoin::TxOut {
            value: op.amount,
            script_pubkey: p2wpk,
        };
        bitcoin::blockdata::transaction::Transaction {
            version: 0,
            lock_time: bitcoin::PackedLockTime(0),
            input: vec![peg_out_input],
            output: vec![peg_out_output],
        }
    }
}
