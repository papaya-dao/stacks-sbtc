use crate::config::{Config, Error as ConfigError, SignerKeys};
use crate::net::{Error as HttpNetError, HttpNet, HttpNetListen, Message, Net, NetListen};
use crate::signing_round::{Error as SigningRoundError, MessageTypes, Signable, SigningRound};
use p256k1::ecdsa;
use serde::Deserialize;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use std::{thread, time};
use wtfrost::Scalar;

// on-disk format for frost save data
#[derive(Clone, Deserialize, Default, Debug)]
pub struct Signer {
    pub config: Config,
    pub signer_id: u32,
}

impl Signer {
    pub fn new(config: Config, signer_id: u32) -> Self {
        Self { config, signer_id }
    }

    pub fn start_p2p_sync(&mut self) -> Result<(), Error> {
        let signer_keys = self.config.signers()?;
        let coordinator_public_key = self.config.coordinator_public_key()?;

        //Create http relay
        let net: HttpNet = HttpNet::new(self.config.http_relay_url.clone());
        let net_queue = HttpNetListen::new(net.clone(), vec![]);
        // thread coordination
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        // start p2p sync
        let id = self.signer_id;
        spawn(move || poll_loop(net_queue, tx, id, signer_keys, coordinator_public_key));

        // listen to p2p messages
        self.start_signing_round(&net, rx)
    }

    fn start_signing_round(&self, net: &HttpNet, rx: Receiver<Message>) -> Result<(), Error> {
        let network_private_key = Scalar::try_from(self.config.network_private_key.as_str())
            .expect("failed to parse network_private_key from config");
        let mut round = SigningRound::from(self);
        loop {
            // Retreive a message from coordinator
            let inbound = rx.recv()?; // blocking
            let outbounds = round.process(inbound.msg)?;
            for out in outbounds {
                let msg = Message {
                    msg: out.clone(),
                    sig: match out {
                        MessageTypes::DkgBegin(msg) | MessageTypes::DkgPrivateBegin(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::DkgEnd(msg) | MessageTypes::DkgPublicEnd(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::DkgQuery(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::DkgQueryResponse(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::DkgPublicShare(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::DkgPrivateShares(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::NonceRequest(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::NonceResponse(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::SignShareRequest(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                        MessageTypes::SignShareResponse(msg) => {
                            msg.sign(&network_private_key).expect("").to_vec()
                        }
                    },
                };
                net.send_message(msg)?;
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Http Network Error: {0}")]
    HttpNetError(#[from] HttpNetError),

    #[error("Signing Round Error: {0}")]
    SigningRoundError(#[from] SigningRoundError),

    #[error("Failed to retrieve message: {0}")]
    RecvError(#[from] mpsc::RecvError),

    #[error("Failed to send message")]
    SendError,

    #[error("Config Error: {0}")]
    ConfigError(#[from] ConfigError),
}

impl From<mpsc::SendError<Message>> for Error {
    fn from(_: mpsc::SendError<Message>) -> Error {
        Error::SendError
    }
}

fn poll_loop(
    mut net: HttpNetListen,
    tx: Sender<Message>,
    id: u32,
    signer_keys: SignerKeys,
    coordinator_public_key: ecdsa::PublicKey,
) -> Result<(), Error> {
    const BASE_TIMEOUT: u64 = 2;
    const MAX_TIMEOUT: u64 = 128;
    let mut timeout = BASE_TIMEOUT;
    loop {
        net.poll(id);
        match net.next_message() {
            None => {
                timeout = if timeout == 0 {
                    BASE_TIMEOUT
                } else if timeout >= MAX_TIMEOUT {
                    MAX_TIMEOUT
                } else {
                    timeout * 2
                };
            }
            Some(m) => {
                timeout = 0;
                if verify_msg(&m, &signer_keys, &coordinator_public_key) {
                    // Only send verified messages down the pipe
                    tx.send(m)?;
                }
            }
        };
        thread::sleep(time::Duration::from_millis(timeout));
    }
}

fn verify_msg(
    m: &Message,
    signer_keys: &SignerKeys,
    coordinator_public_key: &ecdsa::PublicKey,
) -> bool {
    match &m.msg {
        MessageTypes::DkgBegin(msg) | MessageTypes::DkgPrivateBegin(msg) => {
            if !msg.verify(&m.sig, coordinator_public_key) {
                tracing::warn!("Received a DkgPrivateBegin message with an invalid signature.");
                return false;
            }
        }
        MessageTypes::DkgEnd(msg) | MessageTypes::DkgPublicEnd(msg) => {
            if let Some(public_key) = signer_keys.signers.get(&msg.signer_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!("Received a DkgPublicEnd message with an invalid signature.");
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a DkgPublicEnd message with an unknown id: {}",
                    msg.signer_id
                );
                return false;
            }
        }
        MessageTypes::DkgPublicShare(msg) => {
            if let Some(public_key) = signer_keys.key_ids.get(&msg.party_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!("Received a DkgPublicShare message with an invalid signature.");
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a DkgPublicShare message with an unknown id: {}",
                    msg.party_id
                );
                return false;
            }
        }
        MessageTypes::DkgPrivateShares(msg) => {
            if let Some(public_key) = signer_keys.key_ids.get(&msg.key_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!(
                        "Received a DkgPrivateShares message with an invalid signature."
                    );
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a DkgPrivateShares message with an unknown id: {}",
                    msg.key_id
                );
                return false;
            }
        }
        MessageTypes::DkgQuery(msg) => {
            if !msg.verify(&m.sig, coordinator_public_key) {
                tracing::warn!("Received a DkgQuery message with an invalid signature.");
                return false;
            }
        }
        MessageTypes::DkgQueryResponse(msg) => {
            let key_id = msg.public_share.id.id.get_u32();
            if let Some(public_key) = signer_keys.key_ids.get(&key_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!(
                        "Received a DkgQueryResponse message with an invalid signature."
                    );
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a DkgQueryResponse message with an unknown id: {}",
                    key_id
                );
                return false;
            }
        }
        MessageTypes::NonceRequest(msg) => {
            if !msg.verify(&m.sig, coordinator_public_key) {
                tracing::warn!("Received a NonceRequest message with an invalid signature.");
                return false;
            }
        }
        MessageTypes::NonceResponse(msg) => {
            if let Some(public_key) = signer_keys.signers.get(&msg.signer_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!("Received a NonceResponse message with an invalid signature.");
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a NonceResponse message with an unknown id: {}",
                    msg.signer_id
                );
                return false;
            }
        }
        MessageTypes::SignShareRequest(msg) => {
            if !msg.verify(&m.sig, coordinator_public_key) {
                tracing::warn!("Received a SignShareRequest message with an invalid signature.");
                return false;
            }
        }
        MessageTypes::SignShareResponse(msg) => {
            if let Some(public_key) = signer_keys.signers.get(&msg.signer_id) {
                if !msg.verify(&m.sig, public_key) {
                    tracing::warn!(
                        "Received a SignShareResponse message with an invalid signature."
                    );
                    return false;
                }
            } else {
                tracing::warn!(
                    "Received a SignShareResponse message with an unknown id: {}",
                    msg.signer_id
                );
                return false;
            }
        }
    }
    true
}
