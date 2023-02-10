use wtfrost::{
    common::{PolyCommitment, PublicNonce},
    v1::SignatureShare,
    Scalar,
};

///
pub trait Protocol {
    type DkgId;
    type ParticipantId;
    type SignatureId;
}

/// Should be sent by DKG coordinator.
/// In theory, a signer can participate in multiple DKGs and be
/// responsible for multiple parties (DKG and signature shares).
pub struct DkgBegin<P: Protocol> {
    pub dkg_id: P::DkgId,
    pub N: u32,
    pub T: u32,
    /// Must have a size of N
    pub party_to_signer_map: Vec<P::ParticipantId>,
}

/// Should be send by each signer after `DkgBegin`
/// A signer should send multiple events if multiple parties were assigned.
pub struct DkgShare<P: Protocol> {
    pub dkg_id: P::DkgId,
    pub party_id: u32,
    pub public: PolyCommitment,
    /// The size of the vector should be `N-1`.
    /// They are messages for all other parties.
    pub private: Vec<Scalar>,
}

/// Should be sent by a signature aggregator.
/// In theory, a signer can be involved in signing multiple messages at the same time.
pub struct Sign<P: Protocol> {
    pub dkg_id: P::DkgId,
    pub signature_id: P::SignatureId,
    // It's a set of T parties so it can be a vector of bits.
    pub parties: Vec<u32>,
    pub message: Vec<u8>,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Nonce<P: Protocol> {
    pub signature_id: P::DkgId,
    pub party_id: u32,
    pub nonce: PublicNonce,
}

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Share<P: Protocol> {
    pub signature_id: P::SignatureId,
    pub party_id: u32,
    pub share: SignatureShare,
}

pub enum Event<P: Protocol> {
    DkgBegin(DkgBegin<P>),
    DkgShare(DkgShare<P>),
    Sign(Sign<P>),
    Nonce(Nonce<P>),
    Share(Share<P>),
}
