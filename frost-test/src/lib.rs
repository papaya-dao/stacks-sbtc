use wtfrost::{common::{PublicNonce, PolyCommitment}, v1::SignatureShare};

pub type Id = [u32; 8];

/// Should be sent by DKG coordinator.
/// In theory, a signer can hold multiple dkgs and multiple parties.
pub struct NewDkg {
    dkg_id: Id,
    N: u32,
    T: u32,
    /// Should have a size of N
    party_to_signer_map: Vec<Id>,  
}

/// Should be send by each signer after `NewDKG`
/// A signer should send multiple events if multiple parties were assigned.
pub struct DkgPolyCommitment {
    dkg_id: Id,
    party_id: u32,
    value: PolyCommitment,
}

/// Should be sent by a signature aggregator.
/// In theory, a signer can be involved in signing multiple messages at the same time.
pub struct Sign {
    dkg_id: Id,
    message_id: Id,
    // It's a set of T parties so it can be a vector of bits.
    parties: Vec<u32>, 
    message: Vec<u8>,   
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Nonce {
    message_id: Id,
    party_id: u32,
    nonce: PublicNonce,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Share {
    message_id: Id,
    party_id: u32,
    share: SignatureShare,
}

pub enum Event {
    NewDkg(NewDkg),
    DkgPolyCommitment(DkgPolyCommitment),
    Sign(Sign),
    Nonce(Nonce),
    Share(Share),
}
