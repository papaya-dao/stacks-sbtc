#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use frost_signer::signing_round::SigningRound;
    use relay_server::{Call, Method, Response, Server};

    #[test]
    fn template_test() {
        let mut server = Server::default();
        let mut signers = [
            SigningRound::new(7, 10, 0, [0, 1].to_vec()),
            SigningRound::new(7, 10, 0, [2, 3].to_vec()),
            SigningRound::new(7, 10, 0, [4, 5, 6, 7, 8].to_vec()),
            SigningRound::new(7, 10, 0, [10].to_vec()),
        ];
        {
            let request = Method::POST.request(
                "/".to_string(),
                Default::default(),
                "Hello!".as_bytes().to_vec(),
            );
            let response = server.call(request);
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
