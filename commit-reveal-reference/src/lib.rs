use bitcoin::{ScriptBuf, Transaction, Witness};

struct PegInCommit {
    commit_tx: Transaction,
    witness_script: Witness,
}

struct PegOutRequestCommit {
    commit_tx: Transaction,
    witness_script: Witness,
    recipient_scriptPubKey: ScriptBuf,
    fulfillment_fee: u64,
}

pub fn peg_in_commit_tx() -> Transaction {
    todo!();
}

pub fn peg_in_reveal_tx() -> Transaction {
    todo!();
}

pub fn peg_out_request_commit_tx() -> Transaction {
    todo!();
}

pub fn peg_out_request_reveal_tx() -> Transaction {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
