// Code generated using `clarinet run ./scripts/generate-tests.ts`
// Manual edits will be lost.
	
import { Clarinet, Tx, Chain, Account, Block, types } from 'https://deno.land/x/clarinet@v1.5.4/index.ts';
import { assertEquals } from 'https://deno.land/std@0.170.0/testing/asserts.ts';

export { Clarinet, Tx, Chain, types, assertEquals };
export type { Account };

const dirOptions = {strAbbreviateSize: Infinity, depth: Infinity, colors: true};

export function printEvents(block: Block) {
	block.receipts.map(({events}) => events && events.map(event => console.log(Deno.inspect(event, dirOptions))));
}

export const bootstrapContracts = [
	'.sbtc-token',
	'.sbtc-peg-in-processor',
	'.sbtc-peg-out-processor',
	'.sbtc-registry',
	'.sbtc-stacking-pool',
	'.sbtc-testnet-debug-controller',
	'.sbtc-token'
];

export function bootstrap(chain: Chain, deployer: Account) {
	const { receipts } = chain.mineBlock([
		Tx.contractCall(
			`${deployer.address}.sbtc-controller`,
			'upgrade',
			[types.list(bootstrapContracts.map(contract => types.tuple({ contract, enabled: true })))],
			deployer.address
		)
	]);
	receipts[0].result.expectOk().expectList().map(result => result.expectBool(true));
}