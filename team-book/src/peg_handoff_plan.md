# Peg handoff design & implementation plan
This doc serves to gather information relating to the sBTC peg handoff functionality
and consolidate this information into an action plan for delivering it.

## Background

The peg-handoff functionality is defined in SIP-021.
However, much relevant information is scattered accross many places.
The following table aims to gather mentions and prior documentation on the peg-handoff functionality.

| Source | Relevant info on peg-handoff |
|---------|---------|
| [SIP-021](https://github.com/stacksgov/sips/blob/56b73eada5ef1b72376f4a230949297b3edcc562/sips/sip-021/sip-021-trustless-two-way-peg-for-bitcoin.md) | <ul><li>Hand-off protocol</li><li>Wire format</li></ul> |
| [sBTC technical roadmap](https://docs.google.com/spreadsheets/d/1DwLNts95_4olTXKFI1sV7kF47UlvbpmdADfhUtZ9xJA/) | <ul><li>Ensure proper peg wallet balance handoff and incentives for both old and new signer/stacker groups.</li><li> Ensure we have a "balance-based" peg wallet management instead of transaction-based. </li></ul>|
| [mini sBTC](https://docs.google.com/document/d/1R33gZupJg0KsY-vRZYbVFwTHRmq2BCIvyPIVeY0JyGM) | <ul><li>Hand-off procedure in mini sBTC</li></ul> |

## Summary
Peg wallet handoff is the procedure of transferring funds from the peg wallet of one reward cycle to the peg wallet of the next reward cycle.

## Transaction
- **Output 1:** OP_RETURN with the following data
    ```
      0      2  3                  11                                     80
      |------|--|------------------|--------------------------------------|
       magic  op   Reward cycle                     memo
    ```
  - **Opcode** (the `op` byte): `H`

- **Output 2:** The new peg wallet address

## What's needed

1. Wire formats in stacks-blockchain.
2. Something in stacks coordinator to initiate the hand-off?
3. Anything else?
