#[cfg(test)]
mod tests {
    use frost::v1::Signer;
    use hashbrown::HashMap;
    use num_traits::Zero;
    use p256k1::point::Point;
    use rand_core::OsRng;
    use relay_server::{Call, Method, Response, Server};

    #[test]
    fn pure_frost_test() {
        let T = 3;
        let N = 4;
        let mut rng = OsRng::default();
        let mut signers = [
            Signer::new(&[0, 1], N, T, &mut rng),
            Signer::new(&[2], N, T, &mut rng),
            Signer::new(&[3], N, T, &mut rng),
        ];

        // DKG (Distributed Key Generation)
        let A = {
            let A = signers
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
                    if let Err(secret_error) = party.compute_secret(h, &A) {
                        Some((party.id, secret_error))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            if secret_errors.is_empty() {
                Ok(A)
            } else {
                Err(secret_errors)
            }
        }
        .unwrap();

        // signing. Signers: 0 (parties: 0, 1) and 1 (parties: 2)
        {
            //
            let bad_poly_commitments = A.iter()
                .filter_map(|A_i| if A_i.verify() { None } else { Some(A_i.id.id) })
                .collect::<Vec<_>>();
            
            if !bad_poly_commitments.is_empty() {
                panic!("{bad_poly_commitments:?}");
            }

            // TODO: Compute pub key from A
            let key = A.iter().fold(Point::zero(), |key, A_i| key + A_i.A[0]);
           
            // Now we have N, T and key and ready to sign.

            let signers = [&signers[0], &signers[1]];
        }
    }

    fn server_test() {
        let mut server = Server::default();
        {
            let request = Method::POST.request(
                "/".to_string(),
                Default::default(),
                "Hello!".as_bytes().to_vec(),
            );
            let response = server.call(request).unwrap();
            let expected = Response::new(
                200,
                "OK".to_string(),
                Default::default(),
                Default::default(),
            );
            assert_eq!(response, expected);
        }
    }
}
