pub trait BitcoinNode {
    fn broadcast_transaction(&self, tx: &BitcoinTransaction);
}

pub type BitcoinTransaction = String;

pub struct LocalhostBitcoinNode {}

impl BitcoinNode for LocalhostBitcoinNode {
    fn broadcast_transaction(&self, tx: &BitcoinTransaction) {
        todo!()
    }
}
