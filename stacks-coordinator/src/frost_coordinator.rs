use frost_coordinator::coordinator::Coordinator;
use frost_signer::net::NetListen;

pub trait FrostCoordinator {
    fn run_dkg_round(&mut self) -> PublicKey;
    fn sign_message(&mut self, message: &str) -> Signature;
}

// TODO: Define these types
pub type Signature = String;
pub type PublicKey = String;

impl<Network> FrostCoordinator for Coordinator<Network>
where
    Network: NetListen,
{
    fn run_dkg_round(&mut self) -> PublicKey {
        self.run_distributed_key_generation();
    }

    fn sign_message(&mut self, message: &str) -> Signature {
        todo!()
    }
}
