#[cfg(test)]
mod tests {
    use frost::v1::Signer;
    use hashbrown::HashMap;
    use rand_core::OsRng;
    use relay_server::{Call, Method, Response, Server};

    #[test]
    fn template_test() {
        let threshold = 3;
        let total = 4;
        let mut rng = OsRng::default();
        let mut signers = [
            Signer::new(&[0, 1], threshold, total, &mut rng),
            Signer::new(&[2], threshold, total, &mut rng),
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

                    if let Err(secret_error) = party.compute_secret(h, &A) {
                        Some((party.id, secret_error))
                    } else {
                        None
                    }
                }).collect::<HashMap<_, _>>();                

            if secret_errors.is_empty() {
                Ok(A)
            } else {
                Err(secret_errors)
            }
        };
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
