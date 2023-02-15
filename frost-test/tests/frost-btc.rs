use frost_signer::signing_round::SigningRound;

//
#[test]
fn frost_dkg_btc() {
    let T = 3; // Threshold
    let N = 6; // Total
    let signers = vec![
        SigningRound::new(T, N, 1, vec![0, 1]),
        SigningRound::new(T, N, 2, vec![2, 3]),
        SigningRound::new(T, N, 3, vec![4, 5]),
    ];

    // DKG

    // Peg-in: spend a P2PKH utxo and lock it into P2TR output script using the frost public aggregate key.

    // Peg-out: spend the output from the Peg-in tx using frost sign.
}
