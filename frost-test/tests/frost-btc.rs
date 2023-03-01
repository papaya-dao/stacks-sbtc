use bitcoin::consensus::Encodable;
use bitcoin::secp256k1::rand::thread_rng;
use bitcoin::{
    KeyPair, OutPoint, PackedLockTime, SchnorrSighashType, Script, Transaction, XOnlyPublicKey,
};
use hashbrown::HashMap;
use rand_core::OsRng;
use wtfrost::common::{PolyCommitment, Signature};
use wtfrost::errors::AggregatorError;
use wtfrost::{
    common::PublicNonce,
    traits::Signer,
    v1::{self, SignatureAggregator},
    Point,
};

#[test]
fn frost_btc() {
    // Singer setup
    let threshold = 3;
    let total = 4;
    let mut rng = OsRng::default();
    let mut signers = [
        v1::Signer::new(&[0, 1], total, threshold, &mut rng),
        v1::Signer::new(&[2], total, threshold, &mut rng),
        v1::Signer::new(&[3], total, threshold, &mut rng),
    ];

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let user_keys: bitcoin::KeyPair = KeyPair::new(&secp, &mut thread_rng());

    // DKG (Distributed Key Generation)
    let (public_key_shares, group_public_key) = dkg_round(&mut rng, &mut signers);

    // Peg Wallet Address from group key
    let peg_wallet_address =
        bitcoin::PublicKey::from_slice(&group_public_key.compress().as_bytes()).unwrap();

    // Peg in to stx address
    let stx_address = [0; 32];
    let peg_in_step_a = build_peg_in_step_a(1000, peg_wallet_address, stx_address);
    let mut peg_in_step_a_bytes: Vec<u8> = vec![];
    peg_in_step_a
        .consensus_encode(&mut peg_in_step_a_bytes)
        .unwrap();
    println!("peg-in step A tx");
    println!("{:?}", hex::encode(&peg_in_step_a_bytes));

    let peg_in_step_b = build_peg_in_step_b(&peg_in_step_a, peg_wallet_address);
    let mut peg_in_step_b_bytes: Vec<u8> = vec![];
    peg_in_step_b
        .consensus_encode(&mut peg_in_step_b_bytes)
        .unwrap();
    println!("peg-in step B tx");
    println!("{:?}", hex::encode(&peg_in_step_b_bytes));

    // Peg out to btc address
    let public_key_type_transmogrify =
        bitcoin::PublicKey::from_slice(&user_keys.public_key().serialize()).unwrap();
    let mut peg_out = build_peg_out(1000, public_key_type_transmogrify, &peg_in_step_b);
    let mut peg_out_bytes: Vec<u8> = vec![];
    let _peg_out_bytes_len = peg_out.consensus_encode(&mut peg_out_bytes).unwrap();

    let sighash = peg_out.signature_hash(
        0,
        &peg_in_step_b.output[0].script_pubkey,
        SchnorrSighashType::All as u32,
    );
    let signing_payload = sighash.as_hash().to_vec();

    // signing. Signers: 0 (parties: 0, 1) and 1 (parties: 2)
    let result_ok = signing_round(
        &signing_payload,
        threshold,
        total,
        &mut rng,
        &mut signers,
        public_key_shares,
    );
    assert!(result_ok.is_ok());
    let result = result_ok.unwrap();

    let mut sig_bytes = vec![];
    let sig_pubkey_xonly = result.R.compress().as_bytes()[1..32].to_vec();
    sig_bytes.extend(sig_pubkey_xonly);
    sig_bytes.extend(result.z.to_bytes());
    peg_out.input[0].witness.push(&sig_bytes);
    peg_out.input[0]
        .witness
        .push(&group_public_key.compress().as_bytes());
    println!("peg-out tx");
    println!("{:?}", hex::encode(&peg_out_bytes));
}

fn build_peg_in_step_a(
    satoshis: u64,
    peg_wallet_address: bitcoin::PublicKey,
    stx_address: [u8; 32],
) -> Transaction {
    // Peg-In TX
    let peg_in_input = bitcoin::TxIn {
        previous_output: Default::default(),
        script_sig: Default::default(),
        sequence: Default::default(),
        witness: Default::default(),
    };
    let p2wpk = Script::new_v0_p2wpkh(&peg_wallet_address.wpubkey_hash().unwrap());
    let peg_in_output = bitcoin::TxOut {
        value: satoshis,
        script_pubkey: p2wpk,
    };
    bitcoin::blockdata::transaction::Transaction {
        version: 0,
        lock_time: PackedLockTime(0),
        input: vec![peg_in_input],
        output: vec![peg_in_output],
    }
}

fn build_peg_in_step_b(
    step_a: &Transaction,
    peg_wallet_address: bitcoin::PublicKey,
) -> Transaction {
    let peg_in_outpoint = OutPoint {
        txid: step_a.txid(),
        vout: 0,
    };
    let peg_out_input = bitcoin::TxIn {
        previous_output: peg_in_outpoint,
        script_sig: Default::default(),
        sequence: Default::default(),
        witness: Default::default(),
    };
    // crate type weirdness
    let peg_wallet_address_secp =
        bitcoin::secp256k1::PublicKey::from_slice(&peg_wallet_address.to_bytes()).unwrap();
    let secp = bitcoin::util::key::Secp256k1::new();
    let taproot = Script::new_v1_p2tr(&secp, XOnlyPublicKey::from(peg_wallet_address_secp), None);
    let peg_out_output = bitcoin::TxOut {
        value: step_a.output[0].value,
        script_pubkey: taproot,
    };
    bitcoin::blockdata::transaction::Transaction {
        version: 0,
        lock_time: PackedLockTime(0),
        input: vec![peg_out_input],
        output: vec![peg_out_output],
    }
}

fn build_peg_out(
    satoshis: u64,
    user_address: bitcoin::PublicKey,
    utxo: &Transaction,
) -> Transaction {
    let peg_in_outpoint = OutPoint {
        txid: utxo.txid(),
        vout: 0,
    };
    let peg_out_input = bitcoin::TxIn {
        previous_output: peg_in_outpoint,
        script_sig: Default::default(),
        sequence: Default::default(),
        witness: Default::default(),
    };
    let p2wpk = Script::new_v0_p2wpkh(&user_address.wpubkey_hash().unwrap());
    let peg_out_output = bitcoin::TxOut {
        value: satoshis,
        script_pubkey: p2wpk,
    };
    bitcoin::blockdata::transaction::Transaction {
        version: 0,
        lock_time: PackedLockTime(0),
        input: vec![peg_out_input],
        output: vec![peg_out_output],
    }
}

fn signing_round(
    message: &[u8],
    threshold: usize,
    total: usize,
    mut rng: &mut OsRng,
    signers: &mut [v1::Signer; 3],
    public_commitments: Vec<PolyCommitment>,
) -> Result<Signature, AggregatorError> {
    // decide which signers will be used
    let mut signers = [signers[0].clone(), signers[1].clone()];

    // get nonces and shares
    let (nonces, shares) = {
        let ids: Vec<usize> = signers.iter().flat_map(|s| s.get_ids()).collect();
        // get nonces
        let nonces: Vec<PublicNonce> = signers
            .iter_mut()
            .flat_map(|s| s.gen_nonces(&mut rng))
            .collect();
        // get shares
        let shares = signers
            .iter()
            .flat_map(|s| s.sign(message, &ids, &nonces))
            .collect::<Vec<_>>();

        (nonces, shares)
    };

    SignatureAggregator::new(total, threshold, public_commitments.clone())
        .unwrap()
        .sign(&message, &nonces, &shares)
}

fn dkg_round(
    mut rng: &mut OsRng,
    signers: &mut [v1::Signer; 3],
) -> (Vec<PolyCommitment>, wtfrost::Point) {
    {
        let public_key_shares = signers
            .iter()
            .flat_map(|s| s.get_poly_commitments(&mut rng))
            .collect::<Vec<_>>();

        // each party broadcasts their commitments
        // these hashmaps will need to be serialized in tuples w/ the value encrypted
        let broadcast_shares = signers
            .iter()
            .flat_map(|signer| signer.parties.iter())
            .map(|party| (party.id, party.get_shares()))
            .collect::<Vec<_>>();

        // each party collects its shares from the broadcasts
        // maybe this should collect into a hashmap first?
        let secret_errors = signers
            .iter_mut()
            .flat_map(|s| s.parties.iter_mut())
            .filter_map(|party| {
                let h = broadcast_shares
                    .iter()
                    .map(|(id, share)| (*id, share[&party.id]))
                    .collect::<HashMap<_, _>>();

                // should a signer go at error state if error?
                if let Err(secret_error) = party.compute_secret(h, &public_key_shares) {
                    Some((party.id, secret_error))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        if secret_errors.is_empty() {
            let group_key = public_key_shares
                .iter()
                .fold(Point::default(), |s, public_share| s + public_share.A[0]);
            Ok((public_key_shares, group_key))
        } else {
            Err(secret_errors)
        }
    }
    .unwrap()
}
