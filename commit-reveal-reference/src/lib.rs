use std::{
    convert::TryInto,
    iter::{empty, repeat},
};

use bitcoin::{
    absolute::LockTime,
    key::UntweakedPublicKey,
    opcodes::all::{OP_CHECKSIG, OP_DROP, OP_RETURN},
    script::{Builder, PushBytes},
    taproot::{Signature, TaprootBuilder, TaprootSpendInfo},
    Address as BitcoinAddress, Network, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn,
    TxOut, Witness,
};
use blockstack_lib::{codec::StacksMessageCodec, types::chainstate::StacksAddress};
use secp256k1::{ecdsa::RecoverableSignature, XOnlyPublicKey};

pub struct PegInCommitInput {
    pub address: StacksAddress,
    pub amount: u64,
    pub revealer_pub_key: XOnlyPublicKey,
}

pub struct PegInCommitOutput {
    pub address: BitcoinAddress,
    pub witness_script: Witness,
}

pub struct PegInRevealInput {
    pub witness_script: Witness,
    pub commit_output: OutPoint,
}

pub struct PegInRevealOutput(Transaction);

pub struct PegOutRequestCommitInput {
    pub recipient_script_pub_key: ScriptBuf,
    pub amount: u64,
    pub fulfillment_fee: u64,
    pub signature: RecoverableSignature,
    pub revealer_pub_key: XOnlyPublicKey,
}

pub struct PegOutRequestCommitOutput {
    pub address: BitcoinAddress,
    pub witness_script: Witness,
    pub recipient_script_pub_key: ScriptBuf,
    pub fulfillment_fee: u64,
}

pub struct PegOutRequestRevealInput {
    pub witness_script: Witness,
    pub commit_output: OutPoint,
    pub recipient_script_pub_key: ScriptBuf,
    pub fulfillment_fee: u64,
}

pub struct PegOutRequestRevealOutput(Transaction);

pub fn peg_in_commit_tx(_input: PegInCommitInput) -> PegInCommitOutput {
    todo!();
}

pub fn peg_in_reveal_tx(_input: PegInRevealInput) -> Transaction {
    todo!();
}

pub fn peg_out_request_commit_tx(_input: PegOutRequestCommitInput) -> PegOutRequestCommitOutput {
    todo!();
}

pub fn peg_out_request_reveal_tx(_input: PegOutRequestRevealInput) -> Transaction {
    todo!();
}

pub fn peg_in_commit(
    address: StacksAddress,
    contract_name: Option<String>,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    let mut data: Vec<u8> = Vec::with_capacity(86);

    data.push('<' as u8);
    data.extend_from_slice(address.bytes.as_bytes());

    let contract_name_bytes = contract_name
        .map(|contract_name| contract_name.as_bytes().to_vec())
        .unwrap_or_default()
        .into_iter()
        .chain(repeat(0))
        .take(40);

    data.extend(contract_name_bytes);
    data.extend(repeat(&0).take(16)); // memo
    data.extend(reveal_fee.to_be_bytes());

    commit(&data, revealer_key, reclaim_key)
}

pub fn peg_out_request_commit(
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    let mut data: Vec<u8> = Vec::with_capacity(86);
    let empty_memo = [0; 4];

    data.push('>' as u8);
    data.extend(amount.to_be_bytes());

    let (recovery_id, signature_bytes) = signature.serialize_compact();
    data.push(recovery_id.to_i32().try_into().unwrap()); // TODO: Handle errors, though this is infallible
    data.extend_from_slice(&signature_bytes);
    data.extend_from_slice(&empty_memo);
    data.extend_from_slice(&reveal_fee.to_be_bytes());

    commit(&data, revealer_key, reclaim_key)
}

pub fn commit(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    address_from_taproot_spend_info(taproot_spend_info(data, revealer_key, reclaim_key))
}

pub fn reveal(
    commit_output: OutPoint,
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> Transaction {
    let merkle_path = Vec::new(); // TODO: Fill in
    let witness_script = Builder::new().into_script(); // TODO: Figure it out
    let witness = Witness::from_slice(&[witness_script.as_bytes().to_vec(), merkle_path]);

    let tx = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: commit_output,
            script_sig: witness_script.to_owned(),
            sequence: Sequence::MAX,
            witness,
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: Builder::new().push_opcode(OP_RETURN).into_script(),
        }],
    };

    tx
}

fn address_from_taproot_spend_info(spend_info: TaprootSpendInfo) -> BitcoinAddress {
    let secp = secp256k1::Secp256k1::new(); // Impure call

    BitcoinAddress::p2tr(
        &secp,
        spend_info.internal_key(),
        spend_info.merkle_root(),
        Network::Testnet, // TODO: Make sure to make this configurable
    )
}

pub fn taproot_spend_info(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> TaprootSpendInfo {
    let reveal_script = op_drop_script(data, revealer_key);
    let reclaim_script = reclaim_script(reclaim_key);

    let secp = secp256k1::Secp256k1::new(); // Impure call
    let internal_key = internal_key();

    TaprootBuilder::new()
        .add_leaf(1, reveal_script)
        .unwrap() // TODO: Handle error
        .add_leaf(1, reclaim_script)
        .unwrap() // TODO: Handle error
        .finalize(&secp, internal_key)
        .unwrap() // TODO: Handle error
}

fn op_drop_script(data: &[u8], revealer_key: &XOnlyPublicKey) -> ScriptBuf {
    let push_bytes: &PushBytes = data.try_into().unwrap();

    Builder::new()
        .push_slice(push_bytes)
        .push_opcode(OP_DROP)
        .push_x_only_key(revealer_key)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

fn reclaim_script(reclaim_key: &XOnlyPublicKey) -> ScriptBuf {
    Builder::new()
        .push_x_only_key(reclaim_key)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

// Just a point with unknown discrete logarithm.
// We use the hash of the data bytes to compute it.
fn internal_key() -> UntweakedPublicKey {
    // Copied from BIP-0341 at https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
    // The BIP recommends a point lift_x(0x0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0).
    // This hex string is copied from the lift_x argument with the first byte stripped.

    // TODO: Verify that this point is secure
    let internal_key_vec =
        array_bytes::hex2bytes("50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0")
            .unwrap();
    XOnlyPublicKey::from_slice(&internal_key_vec).unwrap() // TODO: Error handling
}

pub trait Reveal {
    type AssociatedData;

    fn extra_outputs(&self, associated_data: Self::AssociatedData) -> Vec<TxOut>;

    fn sign(&self, tx: &Transaction) -> Signature;

    fn reveal_tx(
        &self,
        commit_output: OutPoint,
        witness_script: &Script,
        associated_data: Self::AssociatedData,
    ) -> Transaction {
        let merkle_path = Vec::new(); // TODO: Fill in
        let witness = Witness::from_slice(&[witness_script.as_bytes().to_vec(), merkle_path]);

        let tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: commit_output,
                script_sig: witness_script.to_owned(),
                sequence: Sequence::MAX,
                witness,
            }],
            output: self.extra_outputs(associated_data),
        };

        let signature = self.sign(&tx); // TODO: Where to put it

        tx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn should_create_peg_in_commit_tx() {}

    #[test]
    fn internal_key_works() {
        let key = internal_key();
        println!("Key: {:?}", key);
    }
}
