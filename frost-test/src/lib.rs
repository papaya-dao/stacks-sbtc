use wtfrost::{common::{PublicNonce, PolyCommitment}, v1::SignatureShare};

pub type Id = [u32; 8];

/// Should be sent by DKG coordinator.
pub struct NewDkg {
    dkg_id: Id,
    N: u32,
    T: u32,
    /// Should have a size of N
    party_to_signer_map: Vec<Id>,  
}

/// Should be send by each signer after `NewDKG`
/// A signer should send multiple events if multiple parties where assigned.
pub struct DkgPolyCommitment {
    dkg_id: Id,
    party_id: u32,
    value: PolyCommitment,
}

/// Should be sent by a signature aggregator.
pub struct Sign {
    dkg_id: Id,
    message_id: Id,
    parties: Vec<u32>,    
    message: Vec<u8>,   
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties where assigned.
pub struct Nonce {
    message_id: Id,
    party_id: u32,
    nonce: PublicNonce,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties where assigned.
pub struct Share {
    message_id: Id,
    party_id: u32,
    share: SignatureShare,
}
