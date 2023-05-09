use bitcoin::{Address as BitcoinAddress, ScriptBuf, Transaction, Witness};
use blockstack_lib::types::chainstate::StacksAddress;
use secp256k1::{ecdsa::RecoverableSignature, XOnlyPublicKey};

pub struct PegInCommitInput {
    pub address: StacksAddress,
    pub amount: u64,
    pub revealer_pub_key: XOnlyPublicKey,
}

pub struct PegInCommitOutput {
    pub address: BitcoinAddress,
    pub witness_script: Witness,
}

pub struct PegInRevealInput {
    // TODO
}

pub struct PegInRevealOutput {
    tx: Transaction,
}

pub struct PegOutRequestCommitInput {
    pub recipient_script_pub_key: ScriptBuf,
    pub amount: u64,
    pub fulfillment_fee: u64,
    pub signature: RecoverableSignature,
    pub revealer_pub_key: XOnlyPublicKey,
}

pub struct PegOutRequestCommitOutput {
    pub commit_tx: Transaction,
    pub witness_script: Witness,
    pub recipient_script_pub_key: ScriptBuf,
    pub fulfillment_fee: u64,
}

pub struct PegOutRequestRevealInput {
    // TODO
}

pub struct PegOutRequestRevealOutput {
    tx: Transaction,
}

pub fn peg_in_commit_tx(_input: PegInCommitInput) -> PegInCommitOutput {
    todo!();
}

pub fn peg_in_reveal_tx() -> Transaction {
    todo!();
}

pub fn peg_out_request_commit_tx(_input: PegOutRequestCommitInput) -> PegOutRequestCommitOutput {
    todo!();
}

pub fn peg_out_request_reveal_tx() -> Transaction {
    todo!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
