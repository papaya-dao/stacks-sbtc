The following sequence diagrams illustrate the communication process between DKG signers (Signer 1, Signer 2, ..., Signer N) and the Smart Contract in the Stacks blockchain. It shows the sequence of events and flow of data between the signers, the smart contract, and the blockchain for key registration, signer coordination, message signing, and signature verification. Note that communication between signers is through a P2P Network.

```mermaid
sequenceDiagram
participant Signer1
participant SignerN
participant SmartContract
participant Blockchain

Note over Signer1, Blockchain: Key Registration
Signer1->>SmartContract: Register Public Key 1
SignerN->>SmartContract: Register Public Key N

Note over Signer1, Blockchain: Signer Data Retrieval
Signer1->>Blockchain: Get Stackers
Blockchain-->Signer1: Stackers
SignerN->>Blockchain: Get Stackers
Blockchain-->SignerN: Stackers
Signer1->>SmartContract: Get Stacker Info
SmartContract-->Signer1: Stacker Info
SignerN->>SmartContract: Get Stacker Info
SmartContract-->SignerN: Stacker Info
Note over Signer1, Blockchain: Coordinate Signing Round
Signer1->>Signer1: Determine Coordinator
SignerN->>SignerN: Determine Coordinator
Signer1->>Blockchain: Retrieve Unsigned Transactions
Blockchain-->>Signer1: Unsigned Transactions

Note over Signer1, Blockchain: Signing Transactions
loop for each Pending Transaction
    Signer1->>Signer1: Create Partial Signature 1
    Signer1->>SignerN: Request Signature Share
    SignerN->>Blockchain: Verify Signature Request
    SignerN->>SignerN: Create Partial Signature N
    SignerN->>Signer1: Share Partial Signature N
    Signer1->>Signer1: Verify Partial Signature N
    Signer1->>Signer1: Combine Partial Signatures 1..N

    Note over Signer1, Blockchain: Broadcast Transaction
    Signer1->>Blockchain: Broadcast Signed Transaction
    Blockchain-->>Signer1: Transaction Confirmation
end
```

This diagram shows the sequence of events that occur when a signer receives sign requests for a transaction from the P2P Network. For each transaction, it determines whether to approve or reject it based on the signer's configuration. If the Signer can determine the decision, it signs the transaction and broadcasts its share to the P2P network. If the Signer cannot determine the decision, the Signer API notifies the client UI about the decision failure and requests a manual review.

```mermaid
sequenceDiagram
participant P2PNetwork
participant Signer
participant SignerAPI
participant ClientUI

Note over P2PNetwork, ClientUI: Sign Transaction Request

P2PNetwork-->>Signer: Sign Transaction Request
Signer->>Signer: Determine Approval/Rejection
alt Approval/Rejection Determined
    Signer->>Signer: Sign Approval/Rejection
    Signer->>P2PNetwork: Broadcast Signature Share
else Cannot Determine
    Note over Signer, ClientUI: Notify Client UI
    Signer->>SignerAPI: Notify Decision Failure
    SignerAPI-->>Signer: Acknowledge Notification
    SignerAPI->>ClientUI: Notify Decision Failure
    ClientUI-->>SignerAPI: Acknowledge Notification
end
```


The following sequence diagram shows how a signer server app initializes and interacts with a signer lib, and how it registers and responds to requests via a signer API. The server app relies on the signer lib for cryptographic functions like signing and verifying, while the signer API provides a way for external clients such as a Web Client to interact with the signer server app.

```mermaid
sequenceDiagram
participant SignerServerApp
participant SignerLib
participant SignerAPI
participant WebClient

SignerServerApp->>SignerLib: Initialize Signer Lib
SignerLib-->>SignerServerApp: Signer Lib Initialized

Note over SignerServerApp: Signer Server App Starts

SignerServerApp->>SignerAPI: Register Signer API Endpoints
SignerAPI-->>SignerServerApp: Signer API Endpoints Registered

Note over SignerServerApp: Wait for API Requests

WebClient->>SignerAPI: API Request (e.g., /sign, /verify)
SignerAPI->>SignerServerApp: Incoming API Request
SignerServerApp->>SignerLib: Call Signer Function (e.g., sign, transactions)
SignerLib-->>SignerServerApp: Function Result

SignerServerApp->>SignerAPI: API Response
SignerAPI->>WebClient: Response Sent to WebClient
```