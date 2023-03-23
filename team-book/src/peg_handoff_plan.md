# Peg handoff design & implementation plan
An action plan for delivering sBTC peg hand-off functionality.

## Background

The peg hand-off functionality is defined in SIP-021.
However, much relevant information is scattered accross many places.
The following table aims to gather mentions and prior documentation on the peg hand-off functionality.

| Source | Relevant info on peg hand-off |
|---------|---------|
| [SIP-021](https://github.com/stacksgov/sips/blob/56b73eada5ef1b72376f4a230949297b3edcc562/sips/sip-021/sip-021-trustless-two-way-peg-for-bitcoin.md) | <ul><li>Hand-off protocol</li><li>Wire format</li></ul> |
| [sBTC technical roadmap](https://docs.google.com/spreadsheets/d/1DwLNts95_4olTXKFI1sV7kF47UlvbpmdADfhUtZ9xJA/) | <ul><li>Ensure proper peg wallet balance hand-off and incentives for both old and new signer/stacker groups.</li><li> Ensure we have a "balance-based" peg wallet management instead of transaction-based. </li></ul>|
| [mini sBTC](https://docs.google.com/document/d/1R33gZupJg0KsY-vRZYbVFwTHRmq2BCIvyPIVeY0JyGM) | <ul><li>Hand-off procedure in mini sBTC</li></ul> |

## Summary
Peg wallet hand-off is the procedure of transferring funds from the peg wallet of one reward cycle to the peg wallet of the next reward cycle.

Each PoX reward cycle has a peg wallet, which is constructed and maintained by the set of stackers of that reward cycle.
Once a new reward cycle begins, the previous set of stackers has 100 blocks to hand off the BTC of their peg wallet
to the peg wallet of the new reward cycle. To do this, they will construct a peg hand-off transaction.

## Transaction format
- **Output 1:** OP_RETURN with the following data
    ```
      0      2  3                  11                                     80
      |------|--|------------------|--------------------------------------|
       magic  op   Reward cycle                     memo
    ```
  - **Opcode** (the `op` byte): `H`

- **Output 2:** The new peg wallet address
  - **Amount:** >= Balance of the previous peg-wallet at the end of its reward cycle.

Note: The balance of the peg wallet denotes the total UTXO amount of the peg wallet address.

## Timeline
We need the peg hand-off functionality in place before May 10.

## Tasks

### 1. Peg-handoff wire format in stacks-blockchain.
Following the examples of the peg-in, peg-out-request and peg-out-response wire formats.

### 2. Add support for peg hand-off in stacks coordinator.
This should be very similar to how

Does stacks-coordinator need to figure out when a reward cycle ends? I.e. **when** to trigger a hand-off?
How does this work in mini sBTC?


## TODO: Turn this doc into a top-level GitHub issue