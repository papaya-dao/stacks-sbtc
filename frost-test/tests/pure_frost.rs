use bitcoin::{PackedLockTime, Script, Transaction, XOnlyPublicKey};
use bitcoin::consensus::Encodable;
use bitcoin::secp256k1::PublicKey;
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
fn pure_frost_test() {
    // Singer setup
    let T = 3;
    let N = 4;
    let mut rng = OsRng::default();
    let mut signers = [
        v1::Signer::new(&[0, 1], N, T, &mut rng),
        v1::Signer::new(&[2], N, T, &mut rng),
        v1::Signer::new(&[3], N, T, &mut rng),
    ];

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let user_keys = secp.generate_keypair(&mut OsRng::default());
    //let user_keys = bitcoin::util::key::KeyPair::new(&secp, &mut OsRng::default());

    // DKG (Distributed Key Generation)
    let (public_key_shares, group_key) = dkg_round(&mut rng, &mut signers);

    group_key.compress();
    // Peg Wallet Address from group key
    let peg_wallet_address = bitcoin::secp256k1::PublicKey::from_slice(&group_key.compress().as_bytes()).unwrap();

    // Send to stx address
    let stx_address = [0; 32];
    let peg_in = build_peg_in(1000, peg_wallet_address, stx_address);
    let mut peg_in_bytes: Vec<u8> = vec![];
    peg_in.consensus_encode(&mut peg_in_bytes).unwrap();
    println!("peg-in tx: {:?}", peg_in_bytes);

    let peg_out = build_peg_out(1000, user_keys);
    // signing. Signers: 0 (parties: 0, 1) and 1 (parties: 2)
    let mut peg_out_bytes: Vec<u8> = vec![];
    let peg_out_bytes_len = peg_out.consensus_encode(&mut peg_out_bytes).unwrap();
    let result = signing_round(&peg_out_bytes, T, N, &mut rng, &mut signers, public_key_shares);

    assert!(result.is_ok());
}

fn build_peg_out(
    satoshis: u64,
    user_address: PublicKey,
) -> Transaction {
    bitcoin::blockdata::transaction::Transaction {
        version: 0,
        lock_time: PackedLockTime(0),
        input: vec![],
        output: vec![],
    }
}

fn build_peg_in(
    satoshis: u64,
    peg_wallet_address: PublicKey,
    stx_address: [u8; 32],
) -> Transaction {
    let secp = bitcoin::util::key::Secp256k1::new();

    // Peg-In TX
    let peg_in_input = bitcoin::TxIn {
        previous_output: Default::default(),
        script_sig: Default::default(),
        sequence: Default::default(),
        witness: Default::default(),
    };
    let taproot = Script::new_v1_p2tr(&secp, XOnlyPublicKey::from(peg_wallet_address), None);
    let peg_in_output = bitcoin::TxOut {
        value: satoshis,
        script_pubkey: taproot,
    };
    bitcoin::blockdata::transaction::Transaction {
        version: 0,
        lock_time: PackedLockTime(0),
        input: vec![peg_in_input],
        output: vec![peg_in_output],
    }
}

fn signing_round(
    message: &[u8],
    T: usize,
    N: usize,
    mut rng: &mut OsRng,
    signers: &mut [v1::Signer; 3],
    A: Vec<PolyCommitment>,
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

    SignatureAggregator::new(N, T, A.clone())
        .unwrap()
        .sign(&message, &nonces, &shares)
}

fn dkg_round(mut rng: &mut OsRng, signers: &mut [v1::Signer; 3]) -> (Vec<PolyCommitment>, wtfrost::Point) {
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
