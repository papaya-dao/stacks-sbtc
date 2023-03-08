use serde::Deserialize;
use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use std::{thread, time};
use toml;

use tracing::{debug, info};

use frost_signer::net::{HttpNet, HttpNetError, HttpNetListen, Message, Net, NetListen};
use frost_signer::signing_round::SigningRound;

// maximum party_id
const PARTY_MAX: u32 = 3;

#[derive(Clone, Deserialize, Debug)]
pub struct Signer {
    pub common: Common,
    pub id: u32,
}

#[derive(Clone, Deserialize, Default, Debug)]
pub struct Common {
    pub stacks_node_url: String,
    pub total_signers: usize,
    pub total_parties: usize,
    pub minimum_parties: usize,
}

impl Signer {
    //Create a signer from a given toml filepath.
    //TODO: get config info from sBTC contracts
    pub fn from_file(path: &str) -> Result<Signer, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Invalid path: {}", &e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid toml: {}", e))
    }

    pub fn create_p2p_sync(&mut self) -> Result<(), Error> {
        let net: HttpNet = HttpNet::new(self.common.stacks_node_url.clone());
        // start p2p sync
        let id = self.id;
        let net_queue = HttpNetListen::new(net.clone(), vec![]);

        // thread coordination
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        // Continually poll the network for incoming messages
        spawn(move || poll_network_messages(net_queue, tx, id));

        // listen to p2p messages
        self.sign_received_messages(&net, rx)
    }

    fn sign_received_messages(&self, net: &HttpNet, rx: Receiver<Message>) -> Result<(), Error> {
        assert!(self.id > 0 && self.id <= PARTY_MAX);
        let party_ids = vec![(self.id * 2 - 2) as usize, (self.id * 2 - 1) as usize]; // make two party_ids based on signer_id

        //Create a signing round
        let mut round = SigningRound::new(
            self.common.minimum_parties,
            self.common.total_parties,
            self.id,
            party_ids,
        );

        info!("Signing round beginning for Signer {}", self.id);
        loop {
            let inbound = rx.recv()?; // blocking
            let outbounds = round.process(inbound.msg).map_err(Error::DKGSigningError)?;
            for out in outbounds {
                let msg = Message {
                    msg: out,
                    sig: [0; 32],
                };
                debug!("Signer {} signed message.", self.id);
                net.send_message(msg)?;
                debug!("Signer {} sent signed message.", self.id);
            }
        }
    }
}

fn poll_network_messages(
    mut net: HttpNetListen,
    tx: Sender<Message>,
    id: u32,
) -> Result<(), Error> {
    loop {
        net.poll(id);
        match net.next_message() {
            None => {}
            Some(m) => {
                debug!("Sending message to signer {}: {:?}", id, &m);
                tx.send(m)?;
            }
        };
        thread::sleep(time::Duration::from_millis(500));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Http network error: {0}")]
    HttpNetError(#[from] HttpNetError),
    #[error("Recv Error: {0}")]
    RecvError(#[from] mpsc::RecvError),
    #[error("Send Error")]
    SendError,
    #[error("DKG signing error")]
    DKGSigningError(String),
}

impl From<mpsc::SendError<Message>> for Error {
    fn from(_: mpsc::SendError<Message>) -> Error {
        Error::SendError
    }
}
