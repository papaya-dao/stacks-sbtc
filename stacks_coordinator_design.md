TODO: Create PR for plan to get review

# sBTC private alpha coordinator design
## Overview

The stacks coordinator is a central component of the alpha sBTC testnet. The coordinator is responsible for

- Keeping track of confirmed peg-in & peg-out operations on the Bitcoin chain
- Minting & burning sBTC on the Stacks chain
- Fulfilling peg-out requests

To fulfill the above, the coordinator must:

- Communicate with signers through an http relay to sign peg-out fulfillments and set the peg-wallet address.
- Interact with the alpha sBTC contract to:
  - Set the peg-wallet address.
  - Mint & burn sBTC.
- Poll stacks-node to obtain bitcoin ops.
- Maintain an BTC & STX wallet to cover tx fees.

```mermaid
graph TD
    C[Coordinator]-->|Mint/burn sBTC|D[sBTC contract]
    C-->|Set peg wallet address|D
    C-->|Get sBTC ops|N[Stacks node RPC]
    C-->|DKG+Signing|P[Relay]
    S[Signer]-->|DKG+Signing|P
```

## Design
```rust
struct StacksCoordinator {
  job_queue: JobQueue,
  fee_wallet: FeeWallet,
  frost_coordinator: FrostCoordinator,
}

impl JobQueue {
  fn next_unminted_peg_in_op() -> PegInOp
  fn next_unfulfilled_peg_out_request_op() -> PegOutRequest

  // Additional methods to ensure exactly-once processing of requests may be added
}

// Needs to work on mainnet/testnet stacks and mainnet/testnet bitcoin
impl FeeWallet {
  fn mint_sbtc() -> MintSbtcTransaction
  fn burn_sbtc() -> BurnSbtcTransaction
  fn fulfill_peg_out() -> PegOutFulfillTransaction

  fn set_wallet_address()
}

impl FrostCoordinator {
  fn run_dkg_generation()
  fn sign_message(msg: &str) -> Signature
  fn get_aggregate_public_key() -> SignerPublicKey
}
```

## Implementation plan
TODO: Break out these to separate issues

### #1: Basic structure
Initiate a new project in the `core-eng` repo with the basic structure and components envisioned for the `stacks-coordinator`.
The basic boilerplate should contain:

1. Configuration (Toml)
2. Cli structure (Clap)
3. Mocked types for external interaction points
  - Job Queue
  - Fee wallet
  - Signer coordinator
4. Event loop, incl. tests

Depends on:
- Wire formats

### #2: Job queue
Logic for fetching jobs from the stacks-node RPC endpoint

Depends on:
- #1
- RPC endpoints

### #3: Signer coordinator
Introduce a component to interact with the signers to run distributed key generation & sign messages.

Depends on:
- #1
- frost-signer, frost-coordinator

### #4: Stacks transaction construction
Utilities to generate the necessary transactions to mint & burn sBTC.

Depends on:
- #1
- sBTC contract

### #5: BTC transaction construction
Utilities to generate the bitcion transaction for peg-out fulfillments.

Depends on:
- #1

## Event loop

Question: Do we need to hanlde a dynamic set of signers?

```mermaid
graph TD
    A[Read configuration from file and sBTC contract] --> B[Generate distributed signer key]
    B --> C{Next Bitcoin Op}
    C -->|Peg In| D[Mint sBTC]
    D -->C

    C -->|Peg Out Request| E[Burn sBTC]
    E -->F[Fulfill Peg Out]
    F -->C
```

### Peg out fulfill
```mermaid
    A[]
  
```

## Feature requests
These will be added as standalone issues, not explicitly included in the implementation plan.

### Closed alpha testing
Maintain a closed list of members of the alpha testing. Only addresses in this list is allowed to peg-in & peg-out sBTC.

### One-off commands in CLI
The CLI can start an event loop, but it would also be nice for the coordinator to allow one-off commands such as "process THIS peg-in request" etc.