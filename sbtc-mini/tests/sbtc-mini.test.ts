import { Clarinet, Chain, Account, assertEquals } from "../utils/deps.ts";
import { SbtcMini } from '../models/sbtc-mini.model.ts'
import { Pox2 } from '../models/pox-2.model.ts'

export const ADDRESS = "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM";
export const SBTC_MINI = ADDRESS.concat(".sbtc-mini");

Clarinet.test({
  name: "sbtc-mini: is_authorized_as_stacker() fails if caller is not a stacker",
  fn(chain: Chain, accounts: Map<string, Account>) {
    // arrange
    const deployer = accounts.get("deployer")!;
    const sbtcMini = new SbtcMini(chain, deployer);
    const user = accounts.get("wallet_1")!;

    // act

    // assert
    sbtcMini.is_authorized_as_stacker(user.address, 0).result.expectBool(false);
  }
});

Clarinet.test({
  name: "sbtc-mini: is_authorized_as_stacker() expects contract to have indefinite authorisation to stacker methods for user",
  fn(chain: Chain, accounts: Map<string, Account>) {
    // arrange
    const deployer = accounts.get("deployer")!;
    const sbtcMini = new SbtcMini(chain, deployer);
    const pox2 = new Pox2(chain, deployer);
    const user = accounts.get("wallet_1")!;

    // act
    const { receipts } = chain.mineBlock([
      pox2.allow_contract_caller(user, SBTC_MINI) // pox 2.1 passes none for indefinite/continuous stacking
    ]);
    // console.log('receipts: ', receipts)

    // assert
    assertEquals(receipts.length, 1);
    receipts[0].result.expectOk().expectBool(true);
    sbtcMini.is_authorized_as_stacker(user.address, 0).result.expectBool(true);
  }
});

  Clarinet.test({
    name: "sbtc-mini: get_reward_cycle() returns 0 at first_burn_block_height",
    fn(chain: Chain, accounts: Map<string, Account>) {
      // arrange
      const deployer = accounts.get("deployer")!;
      const sbtcMini = new SbtcMini(chain, deployer);
  
      // act
  
      // assert
      //const expct = Math.floor((SbtcMini.ConstValues.first_burn_block_height - SbtcMini.ConstValues.first_burn_block_height) / SbtcMini.ConstValues.reward_cycle_len);
      sbtcMini.get_reward_cycle(SbtcMini.ConstValues.first_burn_block_height).result.expectUint(0);
    },
  });

