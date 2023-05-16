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

/// Constructs a deposit address for the commit
pub fn commit(
    data: &[u8],
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    let spend_info = taproot_spend_info(data, revealer_key, reclaim_key)?;
    Ok(address_from_taproot_spend_info(spend_info))
}

/// Constructs a transaction that reveals the commit data
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

pub struct RevealInputs<'r> {
    pub commit_output: OutPoint,
    pub stacks_magic_bytes: &'r [u8; 2],
    pub revealer_key: &'r XOnlyPublicKey,
    pub reclaim_key: &'r XOnlyPublicKey,
}

/// Constructs a peg in payment address
pub fn peg_in_commit<'p>(
    peg_in_data: PegInData<'p>,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    commit(&peg_in_data.to_vec(), revealer_key, reclaim_key)
}

/// Constructs a peg out payment address
pub fn peg_out_request_commit(
    peg_out_data: PegOutData,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    commit(&peg_out_data.to_vec()?, revealer_key, reclaim_key)
}

/// Constructs a transaction that reveals the peg in payment address
pub fn peg_in_reveal_unsigned<'p>(
    peg_in_data: PegInData<'p>,
    reveal_inputs: RevealInputs,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let mut tx = reveal(&peg_in_data.to_vec(), reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - peg_in_data.reveal_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

/// Constructs a transaction that reveals the peg out payment address
pub fn peg_out_request_reveal_unsigned(
    peg_out_data: PegOutData,
    reveal_inputs: RevealInputs,
    fulfillment_fee: u64,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
    recipient_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let mut tx = reveal(&peg_out_data.to_vec()?, reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - peg_out_data.reveal_fee - fulfillment_fee,
        script_pubkey: recipient_wallet_address.script_pubkey(),
    });
    tx.output.push(TxOut {
        value: fulfillment_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

pub struct PegInData<'p> {
    pub address: &'p StacksAddress,
    pub contract_name: Option<&'p str>,
    pub reveal_fee: u64,
}

impl<'p> PegInData<'p> {
    pub fn new(
        address: &'p StacksAddress,
        contract_name: Option<&'p str>,
        reveal_fee: u64,
    ) -> Self {
        Self {
            address,
            contract_name,
            reveal_fee,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        once(b'<')
            .chain(once(self.address.version))
            .chain(self.address.bytes.as_bytes().iter().cloned())
            .chain(
                self.contract_name
                    .map(|contract_name| contract_name.as_bytes().to_vec())
                    .into_iter()
                    .flatten(),
            )
            .chain(repeat(0))
            .take(78)
            .chain(self.reveal_fee.to_be_bytes())
            .collect()
    }
}

pub struct PegOutData<'r> {
    pub amount: u64,
    pub signature: &'r RecoverableSignature,
    pub reveal_fee: u64,
}

impl<'r> PegOutData<'r> {
    pub fn new(amount: u64, signature: &'r RecoverableSignature, reveal_fee: u64) -> Self {
        Self {
            amount,
            signature,
            reveal_fee,
        }
    }

    pub fn to_vec(&self) -> CommitRevealResult<Vec<u8>> {
        let (recovery_id, signature_bytes) = self.signature.serialize_compact();
        let recovery_id: u8 = recovery_id
            .to_i32()
            .try_into()
            .map_err(CommitRevealError::InvalidRecoveryId)?;
        let empty_memo = [0; 4];

        Ok(once(b'>')
            .chain(self.amount.to_be_bytes())
            .chain(once(recovery_id))
            .chain(signature_bytes)
            .chain(empty_memo)
            .chain(self.reveal_fee.to_be_bytes())
            .collect())
    }
}

// TODO: Figure out if we want this to be public or not
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
        .cloned()
        .chain(once(b'w'))
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
    use self::helpers::{random_address, random_txid};

    use super::*;

    use bitcoin::{
        address::{Payload::WitnessProgram, WitnessVersion},
        Address, Txid,
    };
    use blockstack_lib::util::hash::Hash160;
    use rand::Rng;
    use secp256k1::{ecdsa::RecoveryId, hashes::Hash};

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

    #[test]
    fn peg_in_commit_should_return_a_valid_bitcoin_p2tr_address() {
        let mut rng = helpers::seeded_rng();
        let revealer_key = helpers::random_key(&mut rng);
        let reclaim_key = helpers::random_key(&mut rng);

        let address = StacksAddress {
            version: 1,
            bytes: Hash160(rng.gen()),
        };

        let commit_address = peg_in_commit(
            PegInData::new(&address, None, 0),
            &revealer_key,
            &reclaim_key,
        )
        .expect("Failed to construct commit address");

        let WitnessProgram(witness_program) = commit_address.payload else {
            panic!("Not a segwit address")
        };

        assert_eq!(witness_program.program().as_bytes().len(), 32);
        assert_eq!(witness_program.version(), WitnessVersion::V1);
    }

    #[test]
    fn peg_out_request_commit_should_return_a_valid_bitcoin_p2tr_address() {
        let mut rng = helpers::seeded_rng();
        let revealer_key = helpers::random_key(&mut rng);
        let reclaim_key = helpers::random_key(&mut rng);

        let signature_bytes: Vec<u8> = std::iter::from_fn(|| rng.gen()).take(64).collect();
        let signature_recovery_id = RecoveryId::from_i32(1).unwrap();

        // TODO: Figure out how to build a good signature
        let signature =
            RecoverableSignature::from_compact(&signature_bytes, signature_recovery_id).unwrap();

        let commit_address = peg_out_request_commit(
            PegOutData::new(100, &signature, 0),
            &revealer_key,
            &reclaim_key,
        )
        .expect("Failed to construct commit address");

        let WitnessProgram(witness_program) = commit_address.payload else {
            panic!("Not a segwit address")
        };

        assert_eq!(witness_program.program().as_bytes().len(), 32);
        assert_eq!(witness_program.version(), WitnessVersion::V1);
    }

    #[test]
    fn peg_in_reveal_unsigned_should_return_a_valid_unsigned_transaction() {
        let mut rng = helpers::seeded_rng();
        let revealer_key = helpers::random_key(&mut rng);
        let reclaim_key = helpers::random_key(&mut rng);
        let peg_wallet_address = random_address(&mut rng);
        let stacks_address = StacksAddress {
            version: 1,
            bytes: Hash160(rng.gen()),
        };

        let commit_output = OutPoint {
            txid: random_txid(&mut rng),
            vout: 0,
        };

        let transaction = peg_in_reveal_unsigned(
            PegInData::new(&stacks_address, None, 40),
            RevealInputs {
                commit_output,
                stacks_magic_bytes: &[b'x', b'x'],
                revealer_key: &revealer_key,
                reclaim_key: &reclaim_key,
            },
            100,
            peg_wallet_address,
        )
        .expect("Couldn't build a reveal transaction");

        assert_eq!(transaction.output.len(), 2);
        assert_eq!(transaction.output[1].value, 60);
    }

    #[test]
    fn peg_out_request_reveal_unsigned_should_return_a_valid_unsigned_transaction() {
        let mut rng = helpers::seeded_rng();
        let revealer_key = helpers::random_key(&mut rng);
        let reclaim_key = helpers::random_key(&mut rng);
        let peg_wallet_address = random_address(&mut rng);
        let recipient_wallet_address = random_address(&mut rng);

        let commit_output = OutPoint {
            txid: random_txid(&mut rng),
            vout: 0,
        };

        let signature_bytes: Vec<u8> = std::iter::from_fn(|| rng.gen()).take(64).collect();
        let signature_recovery_id = RecoveryId::from_i32(1).unwrap();

        // TODO: Figure out how to build a good signature
        let signature =
            RecoverableSignature::from_compact(&signature_bytes, signature_recovery_id).unwrap();

        let transaction = peg_out_request_reveal_unsigned(
            PegOutData::new(100, &signature, 40),
            RevealInputs {
                commit_output,
                stacks_magic_bytes: &[b'x', b'x'],
                revealer_key: &revealer_key,
                reclaim_key: &reclaim_key,
            },
            10,
            100,
            peg_wallet_address,
            recipient_wallet_address,
        )
        .expect("Couldn't build a reveal transaction");

        assert_eq!(transaction.output.len(), 2);
        assert_eq!(transaction.output[1].value, 100);
    }

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
        use bitcoin::{Address, Network, PrivateKey, PublicKey};
        use secp256k1::{KeyPair, Secp256k1, SecretKey};

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

        pub(super) fn random_address<Rng: rand::Rng>(rng: &mut Rng) -> Address {
            let secp = Secp256k1::new();
            let secret_key = SecretKey::new(&mut rand::thread_rng());
            let private_key = PrivateKey::new(secret_key, Network::Regtest);
            let public_key = PublicKey::from_private_key(&secp, &private_key);
            let address = Address::p2wpkh(&public_key, Network::Regtest).unwrap();

            address
        }
    }
}
