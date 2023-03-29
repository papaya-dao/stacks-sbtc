import { Chain, Account, Tx, types, ReadOnlyFn } from "../utils/deps.ts";

interface Extension {
  extension: string;
  enabled: boolean;
}

export class Pox2 {
  // Basic Info
  name = "pox-2";
  chain: Chain;
  deployer: Account;

  constructor(chain: Chain, deployer: Account) {
    this.chain = chain;
    this.deployer = deployer;
  }


  // ---------------------------------------------------------------------------------
  // public methods
  // ---------------------------------------------------------------------------------

  allow_contract_caller(sender: Account, caller: string, burn_ht?: number) {
    return Tx.contractCall(this.name, "allow-contract-caller", [types.principal(caller), (burn_ht) ? types.some(types.uint(burn_ht)) : types.none()], sender.address);
  }

  // ---------------------------------------------------------------------------------
  // read only methods
  // ---------------------------------------------------------------------------------

  get_allowance_contract_callers (stacker: string, calling_contract: string): ReadOnlyFn {
    return this.callReadOnlyFn("get-allowance-contract-callers", [types.principal(stacker), types.principal(calling_contract)]);
  }

  private callReadOnlyFn(method: string, args: Array<any> = [], sender: Account = this.deployer): ReadOnlyFn {
    const regex = /_/g;
    method = method.replace(regex, '-');
    const result = this.chain.callReadOnlyFn(this.name, method, args, sender?.address);
    return result;
  }

}
