use bitcoin::{Address as BitcoinAddress, OutPoint, ScriptBuf, Transaction, TxOut, Witness};
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
    pub witness_script: Witness,
    pub commit_output: OutPoint,
}

pub struct PegInRevealOutput(Transaction);

pub struct PegOutRequestCommitInput {
    pub recipient_script_pub_key: ScriptBuf,
    pub amount: u64,
    pub fulfillment_fee: u64,
    pub signature: RecoverableSignature,
    pub revealer_pub_key: XOnlyPublicKey,
}

pub struct PegOutRequestCommitOutput {
    pub address: BitcoinAddress,
    pub witness_script: Witness,
    pub recipient_script_pub_key: ScriptBuf,
    pub fulfillment_fee: u64,
}

pub struct PegOutRequestRevealInput {
    pub witness_script: Witness,
    pub commit_output: OutPoint,
    pub recipient_script_pub_key: ScriptBuf,
    pub fulfillment_fee: u64,
}

pub struct PegOutRequestRevealOutput(Transaction);

pub fn peg_in_commit_tx(_input: PegInCommitInput) -> PegInCommitOutput {
    todo!();
}

pub fn peg_in_reveal_tx(_input: PegInRevealInput) -> Transaction {
    todo!();
}

pub fn peg_out_request_commit_tx(_input: PegOutRequestCommitInput) -> PegOutRequestCommitOutput {
    todo!();
}

pub fn peg_out_request_reveal_tx(_input: PegOutRequestRevealInput) -> Transaction {
    todo!();
}

pub trait Revealer {
    type ExtraData;

    fn extra_outputs(&self, extra_data: Self::ExtraData) -> Vec<TxOut>;

    fn sign(&self, tx: &mut Transaction);

    fn reveal_tx(commit_output: OutPoint, witness: Witness) -> Transaction {
        // Provided method
        todo!();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn should_create_peg_in_commit_tx() {}
}
