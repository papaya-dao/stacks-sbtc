use wtfrost::common::PublicNonce;

use super::super::Protocol;

/// Should be send by a signer.
/// A signer should send multiple events if multiple parties were assigned.
pub struct Nonce<P: Protocol> {
    pub signature_id: P::DkgId,
    pub party_id: u32,
    pub nonce: PublicNonce,
}
