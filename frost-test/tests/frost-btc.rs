use frost_signer::net::{HttpNet, HttpNetListen, Message, NetListen};
use frost_signer::signing_round::{DkgEnd, MessageTypes, SigningRound};
use std::future::Future;
use std::net::TcpListener;
use std::slice::Iter;
use std::{thread, time};
use std::thread::spawn;

const relay_addr: &str = "127.0.0.1:9776";

//
#[test]
fn frost_dkg_btc() {
    spawn(|| relay_server());

    let t = 3; // Threshold
    let n = 6; // Total

    // Signers
    let mut signers = vec![
        SigningRound::new(t, n, 1, vec![0, 1]),
        SigningRound::new(t, n, 2, vec![2, 3]),
        SigningRound::new(t, n, 3, vec![4, 5]),
    ];

    // Signers networking
    let net: HttpNet = HttpNet::new(relay_addr.to_owned());
    let mut net_queue = HttpNetListen::new(net.clone(), vec![]);

    // DKG

    let mut in_msgs = vec![];
    for signer in signers.iter() {
        net_queue.poll(signer.signer.signer_id);
        match net_queue.next_message() {
            None => {
                println!("signer #{} poll empty", signer.signer.signer_id);
                thread::sleep(time::Duration::from_millis(100)) },
            Some(msg) => in_msgs.push(msg),
        }
    }
    let mut out_msgs = vec![];
    for msg in in_msgs {
        signers = deliver(signers, &msg.msg, &mut out_msgs);
    }

    // Peg-in: spend a P2PKH utxo and lock it into P2TR output script using the frost public aggregate key.

    // Peg-out: spend the output from the Peg-in tx using frost sign.
}

fn relay_server() {
    let listener = TcpListener::bind(relay_addr).unwrap();
    let mut server = relay_server::Server::default();
    for stream_or_error in listener.incoming() {
        let f = || server.update(&mut stream_or_error?);
        if let Err(e) = f() {
            eprintln!("IO error: {e}");
        }
    }
}

fn deliver(
    mut recipients: Vec<SigningRound>,
    msg: &MessageTypes,
    out_msgs: &mut Vec<MessageTypes>,
) -> Vec<SigningRound> {
    for recipient in recipients.iter_mut() {
        let mut new_msgs = recipient.process(msg).unwrap();
        out_msgs.append(&mut new_msgs);
    }
    recipients
}
