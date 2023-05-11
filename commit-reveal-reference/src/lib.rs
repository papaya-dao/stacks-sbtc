use std::{convert::TryInto, iter::repeat};

use bitcoin::{
    absolute::LockTime,
    key::UntweakedPublicKey,
    opcodes::all::{OP_CHECKSIG, OP_DROP, OP_RETURN},
    script::{Builder, PushBytes},
    taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo},
    Address as BitcoinAddress, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Witness,
};
use blockstack_lib::types::chainstate::StacksAddress;
use secp256k1::{ecdsa::RecoverableSignature, XOnlyPublicKey};

fn peg_in_data(address: StacksAddress, contract_name: Option<String>, reveal_fee: u64) -> Vec<u8> {
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

    data
}

fn peg_out_data(amount: u64, signature: RecoverableSignature, reveal_fee: u64) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(86);
    let empty_memo = [0; 4];

    data.push('>' as u8);
    data.extend(amount.to_be_bytes());

    let (recovery_id, signature_bytes) = signature.serialize_compact();
    data.push(recovery_id.to_i32().try_into().unwrap()); // TODO: Handle errors, though this is infallible
    data.extend_from_slice(&signature_bytes);
    data.extend_from_slice(&empty_memo);
    data.extend_from_slice(&reveal_fee.to_be_bytes());

    data
}

pub fn peg_in_commit(
    address: StacksAddress,
    contract_name: Option<String>,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    commit(
        &peg_in_data(address, contract_name, reveal_fee),
        revealer_key,
        reclaim_key,
    )
}

pub fn peg_out_request_commit(
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    commit(
        &peg_out_data(amount, signature, reveal_fee),
        revealer_key,
        reclaim_key,
    )
}

pub fn commit(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> BitcoinAddress {
    address_from_taproot_spend_info(taproot_spend_info(data, revealer_key, reclaim_key))
}

pub fn peg_in_reveal_unsigned(
    address: StacksAddress,
    contract_name: Option<String>,
    reveal_fee: u64,
    commit_output: OutPoint,
    commit_amount: u64,
    stacks_magic_bytes: &[u8; 2],
    peg_wallet_address: BitcoinAddress,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> Transaction {
    let mut tx = reveal(
        commit_output,
        stacks_magic_bytes,
        &peg_in_data(address, contract_name, reveal_fee),
        revealer_key,
        reclaim_key,
    );

    tx.output.push(TxOut {
        value: commit_amount - reveal_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    tx
}

pub fn peg_out_request_reveal_unsigned(
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
    fulfillment_fee: u64,
    commit_output: OutPoint,
    commit_amount: u64,
    stacks_magic_bytes: &[u8; 2],
    peg_wallet_address: BitcoinAddress,
    recipient_wallet_address: BitcoinAddress,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> Transaction {
    let mut tx = reveal(
        commit_output,
        stacks_magic_bytes,
        &peg_out_data(amount, signature, reveal_fee),
        revealer_key,
        reclaim_key,
    );

    tx.output.push(TxOut {
        value: commit_amount - reveal_fee - fulfillment_fee,
        script_pubkey: recipient_wallet_address.script_pubkey(),
    });
    tx.output.push(TxOut {
        value: fulfillment_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    tx
}

pub fn reveal(
    commit_output: OutPoint,
    stacks_magic_bytes: &[u8; 2],
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> Transaction {
    let spend_info = taproot_spend_info(data, revealer_key, reclaim_key);

    let script = op_drop_script(data, revealer_key);
    let control_block = spend_info
        .control_block(&(script.clone(), LeafVersion::TapScript))
        .unwrap(); // TODO: Handle none

    let witness_script = Builder::new().into_script(); // TODO: Figure it out
    let witness = Witness::from_slice(&[script.as_bytes().to_vec(), control_block.serialize()]);

    let reveal_op_return_bytes: Vec<u8> = stacks_magic_bytes
        .iter()
        .chain(&['w' as u8])
        .cloned()
        .collect();

    let reveal_op_return_pushbytes: &PushBytes =
        reveal_op_return_bytes.as_slice().try_into().unwrap(); // TODO: Error handling

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
            script_pubkey: Builder::new()
                .push_opcode(OP_RETURN)
                .push_slice(reveal_op_return_pushbytes)
                .into_script(),
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
