use std::net::TcpListener;
use std::thread::spawn;
use frost_signer::net::{HttpNet, HttpNetListen};
use frost_signer::signing_round::{MessageTypes, SigningRound};

//
#[test]
fn frost_dkg_btc() {
    spawn (|| { relay_server() });

    let t = 3; // Threshold
    let n = 6; // Total
    let signers = vec![
        SigningRound::new(t, n, 1, vec![0, 1]),
        SigningRound::new(t, n, 2, vec![2, 3]),
        SigningRound::new(t, n, 3, vec![4, 5]),
    ];

    // DKG
    let net: HttpNet = HttpNet::new(config.common.stacks_node_url.clone());

    //broadcast(signers, msg);
    // Peg-in: spend a P2PKH utxo and lock it into P2TR output script using the frost public aggregate key.

    // Peg-out: spend the output from the Peg-in tx using frost sign.
}

fn next_message(mut net: HttpNet,  id: u32) {
    net.poll(id);
    match net.next_message() {
        None => {
            thread::sleep(time::Duration::from_millis(500));
        }
        Some(m) => {
            tx.send(m).unwrap();
        }
    };

}

fn relay_server() {
    let addr = "127.0.0.1:9776";
    let listener = TcpListener::bind(addr).unwrap();
    let mut server = relay_server::Server::default();
    for stream_or_error in listener.incoming() {
        let f = || server.update(&mut stream_or_error?);
        if let Err(e) = f() {
            eprintln!("IO error: {e}");
        }
    }
}

fn broadcast(recipients: Vec<SigningRound>, msg: MessageTypes) {
    let mut out_queue : Vec<MessageTypes> =  vec![];
    for mut recipient in recipients {
        let mut out_msgs = recipient.process(&msg).unwrap();
        out_queue.append(&mut out_msgs);
    }
}
