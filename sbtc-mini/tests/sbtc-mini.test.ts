import { Clarinet, Chain, Account } from "../utils/deps.ts";
import { SbtcMini } from '../models/sbtc-mini.model.ts'

Clarinet.test({
  name: "sbtc-mini: is_authorized_as_stacker() fails if caller is not DAO or extension",
  fn(chain: Chain, accounts: Map<string, Account>) {
    // arrange
    const deployer = accounts.get("deployer")!;
    const sbtcMini = new SbtcMini(chain, deployer);
    const user = accounts.get("wallet_1")!;

    // act

    // assert
    sbtcMini.is_authorized_as_stacker(user.address, 0).result.expectBool(false);
  },
});

