// Code generated using `clarinet run ./scripts/generate-tests.ts`
// Manual edits will be lost.

import { Clarinet, Tx, Chain, Account, types, assertEquals, printEvents, bootstrap } from './deps.ts';

Clarinet.test({
	name: "test-sign-pre-register-early: user can pre-register",
	async fn(chain: Chain, accounts: Map<string, Account>) {
		const deployer = accounts.get("deployer")!;
		bootstrap(chain, deployer);
		let callerAddress = accounts.get('wallet_1')!.address;
		let block;
		
			  block = chain.mineBlock([Tx.contractCall("pox-3", "mock-set-stx-account", [types.principal("ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5"), types.tuple({"locked": types.uint(1000000000), "unlock-height": types.uint(100), "unlocked": types.uint(100000000000)})], callerAddress)
  ,
	
	  ]);
	  block.receipts.map(({result}) => result.expectOk());
	  
	}
});
