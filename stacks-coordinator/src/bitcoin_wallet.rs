use crate::bitcoin_node::BitcoinTransaction;
use crate::peg_wallet::{BitcoinWallet as BitcoinWalletTrait, Error as PegWalletError};
use crate::stacks_node::PegOutRequestOp;
use bitcoin::{hashes::Hash, Script, WPubkeyHash};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("type conversion error from blockstack::bitcoin to bitcoin:: {0}")]
    ConversionError(#[from] bitcoin::hashes::Error),
    #[error("type conversion error blockstack::bitcoin::hashes:hex {0}")]
    ConversionErrorHex(#[from] bitcoin::hashes::hex::Error),
}

pub struct BitcoinWallet {}

fn build_transaction(op: &PegOutRequestOp) -> Result<BitcoinTransaction, Error> {
    let utxo = bitcoin::OutPoint {
        txid: bitcoin::Txid::from_slice(&[0; 32])?,
        vout: op.vtxindex,
    };
    let peg_out_input = bitcoin::TxIn {
        previous_output: utxo,
        script_sig: Default::default(),
        sequence: Default::default(),
        witness: Default::default(),
    };
    let user_address_hash = bitcoin::hashes::hash160::Hash::from_slice(&op.recipient.bytes())?;
    let recipient_p2wpk = Script::new_v0_p2wpkh(&WPubkeyHash::from_hash(user_address_hash));
    let peg_out_output_recipient = bitcoin::TxOut {
        value: op.amount,
        script_pubkey: recipient_p2wpk,
    };
    let change_address_p2tr = Script::default(); // todo: Script::new_v1_p2tr();
    let peg_out_output_change = bitcoin::TxOut {
        value: op.amount,
        script_pubkey: change_address_p2tr,
    };
    Ok(bitcoin::blockdata::transaction::Transaction {
        version: 2,
        lock_time: bitcoin::PackedLockTime(0),
        input: vec![peg_out_input],
        output: vec![peg_out_output_recipient, peg_out_output_change],
    })
}

impl BitcoinWalletTrait for BitcoinWallet {
    type Error = Error;
    fn fulfill_peg_out(&self, op: &PegOutRequestOp) -> Result<BitcoinTransaction, PegWalletError> {
        let tx = build_transaction(op)?;
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::BitcoinWallet;
    use crate::peg_wallet::BitcoinWallet as BitcoinWalletTrait;

    use blockstack_lib::{
        burnchains::Txid,
        chainstate::stacks::address::{PoxAddress, PoxAddressType20},
        types::chainstate::BurnchainHeaderHash,
        util::secp256k1::MessageSignature,
    };

    use crate::stacks_node::PegOutRequestOp;

    #[test]
    fn fulfill_peg_out() {
        let wallet = BitcoinWallet {};
        let bitcoin_address =
            bitcoin::hashes::hex::FromHex::from_hex("dbc67065ff340e44956471a4b85a6b636c223a06")
                .unwrap();
        let recipient = PoxAddress::Addr20(true, PoxAddressType20::P2WPKH, bitcoin_address);
        let peg_wallet_address = PoxAddress::Addr20(true, PoxAddressType20::P2WPKH, [0x01; 20]);
        let req_op = PegOutRequestOp {
            amount: 1000,
            recipient: recipient,
            signature: MessageSignature([0x00; 65]),
            peg_wallet_address: peg_wallet_address,
            fulfillment_fee: 0,
            memo: vec![],
            txid: Txid([0x04; 32]),
            vtxindex: 0,
            block_height: 0,
            burn_header_hash: BurnchainHeaderHash([0x00; 32]),
        };
        let btc_tx = wallet.fulfill_peg_out(&req_op).unwrap();
        assert_eq!(btc_tx.output[0].value, 1000);
    }
}
