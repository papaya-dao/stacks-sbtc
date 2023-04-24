# sBTC Mini protocol

Bootstrap:

```clarity
(contract-call? .sbtc-controller upgrade (list {contract: .sbtc-token, enabled: true} {contract: .sbtc-peg-in-processor, enabled: true} {contract: .sbtc-peg-out-processor, enabled: true} {contract: .sbtc-registry, enabled: true} {contract: .sbtc-stacking-pool, enabled: true} {contract: .sbtc-token, enabled: true}))
```
