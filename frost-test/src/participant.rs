use std::collections::HashMap;

use rand_core::{CryptoRng, RngCore};
use wtfrost::v1;

use crate::event::{DkgPolyCommitment, Event, Id, DkgBegin};

trait Participant {
    fn call(&mut self, event: &Event) -> Vec<Event>;
}

struct DkgState {
    signer: v1::Signer,
}

struct MemParticipant<R: CryptoRng + RngCore> {
    id: Id,
    rng: R,
    dkg: HashMap<Id, DkgState>,
}

impl<R: CryptoRng + RngCore> MemParticipant<R> {
    fn dkg_begin(&mut self, d: &DkgBegin) -> Vec<Event> {
        let parties = d
            .party_to_signer_map
            .iter()
            .enumerate()
            .filter_map(|(party_id, participant_id)| {
                if *participant_id == self.id {
                    Some(party_id as usize)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if parties.is_empty() {
            return Vec::default();
        }
        let r = &mut self.rng;
        let signer = v1::Signer::new(&parties, d.N as usize, d.T as usize, r);
        let dkg_id = d.dkg_id;
        let events = signer
            .get_poly_commitments(r)
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                Event::DkgPolyCommitment(DkgPolyCommitment {
                    dkg_id,
                    party_id: parties[i] as u32,
                    value,
                })
            })
            .collect::<Vec<_>>();
        self.dkg.insert(dkg_id, DkgState { signer });
        events
    }
}

impl<R: CryptoRng + RngCore> Participant for MemParticipant<R> {
    fn call(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::DkgBegin(d) => self.dkg_begin(d),
            _ => todo!(),
        }
    }
}
