use wtfrost::{
    common::{PolyCommitment, PublicNonce},
    v1::SignatureShare,
    Scalar,
};

// Each signer, DKG and message have a unique id.
pub type Id = [u32; 8];

/// Should be sent by DKG coordinator.
/// In theory, a signer can participate in multiple DKGs and be
/// responsible for multiple parties (DKG and signature shares).
pub struct DkgBegin {
    pub dkg_id: Id,
    pub N: u32,
    pub T: u32,
    /// Must have a size of N
    pub party_to_signer_map: Vec<Id>,
}

/// Should be send by each signer after `NewDKG`
/// A signer should send multiple events if multiple parties were assigned.
pub struct DkgPolyCommitment {
    pub dkg_id: Id,
    pub party_id: u32,
    pub value: PolyCommitment,
}

/// Should be send by each signer after `NewDKG`
/// A signer should send multiple events if multiple parties were assigned.
pub struct DkgShare {
    pub dkg_id: Id,
    pub party_id: u32,
    pub value: Vec<(u32, Scalar)>,
}

/// Should be sent by a signature aggregator.
/// In theory, a signer can be involved in signing multiple messages at the same time.
pub struct Sign {
    pub dkg_id: Id,
    pub signature_id: Id,
    // It's a set of T parties so it can be a vector of bits.
    pub parties: Vec<u32>,
    pub message: Vec<u8>,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Nonce {
    pub signature_id: Id,
    pub party_id: u32,
    pub nonce: PublicNonce,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Share {
    pub message_id: Id,
    pub party_id: u32,
    pub share: SignatureShare,
}

pub enum Event {
    DkgBegin(DkgBegin),
    DkgPolyCommitment(DkgPolyCommitment),
    DkgShare(DkgShare),
    Sign(Sign),
    Nonce(Nonce),
    Share(Share),
}
