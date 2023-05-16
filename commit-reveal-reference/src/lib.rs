// TODO: Before PR
// - [x] Error handling
// - Unit testing
// - Rustdoc for functions and library

// Follow-up
// - Bitcoind integration tests
//  - Run bitcoin node, submit commit transaction

use std::{convert::TryInto, iter::once, iter::repeat, num::TryFromIntError};

use bitcoin::{
    absolute::LockTime,
    key::UntweakedPublicKey,
    opcodes::all::{OP_CHECKSIG, OP_DROP, OP_RETURN},
    script::{Builder, PushBytes},
    taproot::{LeafVersion, TaprootBuilder, TaprootBuilderError, TaprootSpendInfo},
    Address as BitcoinAddress, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Witness,
};
use blockstack_lib::types::chainstate::StacksAddress;
use secp256k1::{ecdsa::RecoverableSignature, XOnlyPublicKey};

type CommitRevealResult<T> = Result<T, CommitRevealError>;

#[derive(thiserror::Error, Debug)]
pub enum CommitRevealError {
    #[error("Signature is invalid: {0}")]
    InvalidRecoveryId(TryFromIntError),
    #[error("Control block could not be built from script")]
    NoControlBlock,
    #[error("Could not build taproot spend info: {0}: {1}")]
    InvalidTaproot(&'static str, TaprootBuilderError),
}

pub fn commit(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    let spend_info = taproot_spend_info(data, revealer_key, reclaim_key)?;
    Ok(address_from_taproot_spend_info(spend_info))
}

pub struct RevealInputs<'r> {
    pub commit_output: OutPoint,
    pub stacks_magic_bytes: &'r [u8; 2],
    pub revealer_key: &'r XOnlyPublicKey,
    pub reclaim_key: &'r XOnlyPublicKey,
}

pub fn reveal(
    data: &[u8],
    RevealInputs {
        commit_output,
        stacks_magic_bytes,
        revealer_key,
        reclaim_key,
    }: RevealInputs,
) -> CommitRevealResult<Transaction> {
    let spend_info = taproot_spend_info(data, revealer_key, reclaim_key)?;

    let script = op_drop_script(data, revealer_key);
    let control_block = spend_info
        .control_block(&(script.clone(), LeafVersion::TapScript))
        .ok_or(CommitRevealError::NoControlBlock)?;

    let witness = Witness::from_slice(&[script.as_bytes().to_vec(), control_block.serialize()]);

    let tx = Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: commit_output,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness,
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: reveal_op_return_script(stacks_magic_bytes),
        }],
    };

    Ok(tx)
}

pub fn peg_in_commit(
    address: StacksAddress,
    contract_name: Option<String>,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    let data = peg_in_data(address, contract_name, reveal_fee);
    Ok(commit(&data, revealer_key, reclaim_key)?)
}

pub fn peg_out_request_commit(
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    let data = peg_out_data(amount, signature, reveal_fee)?;
    let address = commit(&data, revealer_key, reclaim_key)?;

    Ok(address)
}

pub fn peg_in_reveal_unsigned(
    reveal_inputs: RevealInputs,
    address: StacksAddress,
    contract_name: Option<String>,
    reveal_fee: u64,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let data = peg_in_data(address, contract_name, reveal_fee);
    let mut tx = reveal(&data, reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - reveal_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

pub fn peg_out_request_reveal_unsigned(
    reveal_inputs: RevealInputs,
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
    fulfillment_fee: u64,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
    recipient_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let data = peg_out_data(amount, signature, reveal_fee)?;
    let mut tx = reveal(&data, reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - reveal_fee - fulfillment_fee,
        script_pubkey: recipient_wallet_address.script_pubkey(),
    });
    tx.output.push(TxOut {
        value: fulfillment_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

fn peg_in_data(address: StacksAddress, contract_name: Option<String>, reveal_fee: u64) -> Vec<u8> {
    once('<' as u8)
        .chain(once(address.version))
        .chain(address.bytes.as_bytes().into_iter().cloned())
        .chain(
            contract_name
                .map(|contract_name| contract_name.as_bytes().to_vec())
                .into_iter()
                .flatten(),
        )
        .chain(repeat(0))
        .take(78)
        .chain(reveal_fee.to_be_bytes())
        .collect()
}

fn peg_out_data(
    amount: u64,
    signature: RecoverableSignature,
    reveal_fee: u64,
) -> CommitRevealResult<Vec<u8>> {
    let (recovery_id, signature_bytes) = signature.serialize_compact();
    let recovery_id: u8 = recovery_id
        .to_i32()
        .try_into()
        .map_err(CommitRevealError::InvalidRecoveryId)?;
    let empty_memo = [0; 4];

    Ok(once('>' as u8)
        .chain(amount.to_be_bytes())
        .chain(once(recovery_id))
        .chain(signature_bytes)
        .chain(empty_memo)
        .chain(reveal_fee.to_be_bytes())
        .collect())
}

pub fn taproot_spend_info(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<TaprootSpendInfo> {
    let reveal_script = op_drop_script(data, revealer_key);
    let reclaim_script = reclaim_script(reclaim_key);

    let secp = secp256k1::Secp256k1::new(); // Impure call
    let internal_key = internal_key();

    Ok(TaprootBuilder::new()
        .add_leaf(1, reveal_script)
        .map_err(|err| CommitRevealError::InvalidTaproot("Invalid reveal script", err))?
        .add_leaf(1, reclaim_script)
        .map_err(|err| CommitRevealError::InvalidTaproot("Invalid reclaim script", err))?
        .finalize(&secp, internal_key)
        // TODO: Confirm that this is infallible
        .expect("Taproot builder should be able to finalize after adding the internal key"))
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

fn reveal_op_return_script(stacks_magic_bytes: &[u8; 2]) -> ScriptBuf {
    let op_return_bytes: Vec<u8> = stacks_magic_bytes
        .iter()
        .chain(&['w' as u8])
        .cloned()
        .collect();

    // TODO: Confirm that this is infallible
    let op_return_pushbytes: &PushBytes = op_return_bytes.as_slice().try_into().unwrap();

    Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(op_return_pushbytes)
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

    // TODO: Confirm that this is infallible
    XOnlyPublicKey::from_slice(&internal_key_vec).expect("Could not build internal key")
}

#[cfg(test)]
mod tests {
    use super::*;

    use bitcoin::{
        address::{Payload::WitnessProgram, WitnessVersion},
        Txid,
    };
    use rand::Rng;

    #[test]
    fn commit_should_return_a_valid_bitcoin_p2tr_address() {
        let mut rng = helpers::seeded_rng();
        let data = [rng.gen(); 86];
        let revealer_key = helpers::random_key(&mut rng);
        let reclaim_key = helpers::random_key(&mut rng);

        let commit_address =
            commit(&data, &revealer_key, &reclaim_key).expect("Failed to construct commit address");

        let WitnessProgram(witness_program) = commit_address.payload else {
            panic!("Not a segwit address")
        };

        assert_eq!(witness_program.program().as_bytes().len(), 32);
        assert_eq!(witness_program.version(), WitnessVersion::V1);
    }

    #[test]
    fn reveal_should_return_a_valid_unsigned_transaction() {
        let mut rng = helpers::seeded_rng();
        let txid: Txid = helpers::random_txid(&mut rng);
        let commit_output = OutPoint { txid, vout: 0 };
        let data = [rng.gen(); 86];
        let stacks_magic_bytes = &[105, 100]; // "id" - arbitrary but consistent with Stacks tests
        let revealer_key = &helpers::random_key(&mut rng);
        let reclaim_key = &helpers::random_key(&mut rng);

        let reveal_transaction_unsigned = reveal(
            &data,
            RevealInputs {
                commit_output,
                stacks_magic_bytes,
                revealer_key,
                reclaim_key,
            },
        )
        .expect("Failed to construct reveal transaction");

        assert_eq!(reveal_transaction_unsigned.input.len(), 1);
        assert!(reveal_transaction_unsigned.input[0].script_sig.is_empty());
        assert_eq!(reveal_transaction_unsigned.input[0].witness.len(), 2);

        assert_eq!(reveal_transaction_unsigned.output.len(), 1);
        assert_eq!(
            reveal_transaction_unsigned.output[0].script_pubkey,
            reveal_op_return_script(&stacks_magic_bytes)
        );
    }

    // #[test]
    // fn peg_in_commit_should_return_a_valid_bitcoin_p2tr_address() {
    //     assert!(false);
    // }

    //#[test]
    //fn peg_out_request_commit_should_return_a_valid_bitcoin_p2tr_over_p2sh_address() {
    //    assert!(false);
    //}

    //#[test]
    //fn peg_in_reveal_unsigned_should_return_a_valid_unsigned_transaction() {
    //    assert!(false);
    //}

    //#[test]
    //fn peg_out_request_reveal_unsigned_should_return_a_valid_unsigned_transaction() {
    //    assert!(false);
    //}

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

    mod helpers {
        use secp256k1::KeyPair;

        use super::*;

        pub(super) fn seeded_rng() -> rand::rngs::StdRng {
            rand::SeedableRng::from_seed([0; 32])
        }

        // May panic if the randomly generated key is invalid. This should be unlikely but possible.
        pub(super) fn random_key<Rng: rand::Rng>(rng: &mut Rng) -> XOnlyPublicKey {
            let secp = secp256k1::Secp256k1::new();
            let keypair = KeyPair::new(&secp, rng);
            keypair.x_only_public_key().0
        }

        pub(super) fn random_txid<Rng: rand::Rng>(rng: &mut Rng) -> Txid {
            use bitcoin::hashes::sha256d;
            use bitcoin::hashes::Hash;

            let random_hash: sha256d::Hash = Hash::from_byte_array([rng.gen(); 32]);
            random_hash.into()
        }
    }
}
