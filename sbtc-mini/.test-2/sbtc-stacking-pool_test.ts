// Code generated using `clarinet run ./scripts/generate-tests.ts`
// Manual edits will be lost.

import { Clarinet, Tx, Chain, Account, types, assertEquals, printEvents, bootstrap } from './deps.ts';

Clarinet.test({
                  name: "test-pre-register",
                  async fn(chain: Chain, accounts: Map<string, Account>) {
                      const deployer = accounts.get("deployer")!;
                      bootstrap(chain, deployer);
let block0 = chain.mineBlock([
                                      Tx.contractCall("pox-3", "allow-contract-caller", [".sbtc-stacking-pool"], deployer.address)
                                  ]);
chain.mineEmptyBlock(70200);
let block1 = chain.mineBlock([
                                      Tx.contractCall("sbtc-stacking-pool", "signer-pre-register", [types.uint(100000000000)], deployer.address)
                                  ]);
block1.receipts.map(({result}) => result.expectOk());
    }
                            });