use crate::event::{Event, Protocol};

trait Participant: Protocol {
    type P: Protocol;
    fn call(&mut self, event: &Event<Self::P>) -> Vec<Self::P>;
}
