use bitcoin::{
    absolute::LockTime,
    opcodes::all::OP_DROP,
    script::{Builder, PushBytes},
    Address as BitcoinAddress, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Witness,
};
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

fn op_drop_script(data: &[u8]) -> ScriptBuf {
    let push_bytes: &PushBytes = data.try_into().unwrap();
    let script = Builder::new().push_slice(push_bytes).into_script();
    todo!();
}

pub trait Reveal {
    type AssociatedData;

    fn extra_outputs(&self, associated_data: Self::AssociatedData) -> Vec<TxOut>;

    fn sign(&self, tx: &mut Transaction);

    fn reveal_tx(
        &self,
        commit_output: OutPoint,
        witness_script: &Script,
        associated_data: Self::AssociatedData,
    ) -> Transaction {
        // TODO Figure out the correct way to produce a script
        let script_sig = Builder::new()
            .push_slice(&[0x00, 0x00, 0x00, 0x00]) // data
            .push_opcode(OP_DROP)
            .push_slice(&[0x00, 0x00, 0x00, 0x00]) // lock script
            .into_script();

        let merkle_path = Vec::new(); // TODO: Fill in

        let witness = Witness::from_slice(&[witness_script.as_bytes().to_vec(), merkle_path]);

        let mut tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: commit_output,
                script_sig,
                sequence: Sequence::MAX,
                witness,
            }],
            output: vec![],
        };

        tx.output.extend(self.extra_outputs(associated_data));
        self.sign(&mut tx);

        tx
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
