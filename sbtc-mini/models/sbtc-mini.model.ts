import { Chain, Account, Tx, types, ReadOnlyFn } from "../utils/deps.ts";

enum ErrCode {
  ERR_MISSING_DISBURSEMENTS = 1,
  ERR_TX_ALREADY_PROCESSED = 2,
  ERR_INVALID_BITCOIN_TX = 3,
  ERR_INVALID_DISBURSEMENT_HASH = 4,
  
  ERR_NOT_REGISTRATION_WINDOW = 1000,
  ERR_ALREADY_REGISTERED = 1001,
  ERR_SBTC_NOT_AUTHORIZED_STACKER_DELEGATE = 1002,
  ERR_INSUFFICIENT_UNLOCK_BALANCE = 1003,
  ERR_SIGNING_KEY_ALREADY_USED = 1004,
  ERR_NOT_A_STACKER = 1005,
  ERR_STACKER_DOES_NOT_UNLOCK = 1005,
  ERR_STACKER_NOT_QUALIFIED = 1006,
  ERR_STACKER_ALREADY_PRE_REGISTERED = 1007,
  ERR_PAYOUT_ADDRESS_ALREADY_USED = 1008,
  
  ERR_NOT_VOTING_WINDOW = 2000,
  ERR_SIGNER_NOT_REGISTERED = 2002,
  ERR_SIGNER_ALREADY_VOTED = 2003,
  ERR_NOT_ENOUGH_STX = 2004,
  
  ERR_NO_WINNING_POX_ADDR = 3000,
  ERR_FUTURE_REWARD_CYCLE = 3001,
  ERR_PAYOUT_NOT_CALCULATED = 3002,
  ERR_PAYOUT_ALREADY_MADE = 3003,
  ERR_NOT_A_SIGNER = 3004,
}

interface Extension {
  extension: string;
  enabled: boolean;
}

export class SbtcMini {
  // Basic Info

  name = "sbtc-mini";
  static readonly ErrCode = ErrCode;
  chain: Chain;
  deployer: Account;

  constructor(chain: Chain, deployer: Account) {
    this.chain = chain;
    this.deployer = deployer;
  }


  // ---------------------------------------------------------------------------------
  // public methods
  // ---------------------------------------------------------------------------------

  update_btc_payout(sender: Account, rc: number) {
    return Tx.contractCall(this.name, "update-btc-payout", [types.uint(rc)], sender.address);
  }
  process_disbursement_tx(sender: Account, burn_ht: number) {
    return Tx.contractCall(this.name, "process-disbursement-tx", [types.uint(burn_ht)], sender.address);
  }
  signer_vote(sender: Account, amount_ustx: number) {
    return Tx.contractCall(this.name, "signer-vote", [types.uint(amount_ustx)], sender.address);
  }
  signer_register(sender: Account, signer: string) {
    return Tx.contractCall(this.name, "signer-register", [types.principal(signer)], sender.address);
  }
  signer_pre_register(sender: Account, amount_ustx: number) {
    return Tx.contractCall(this.name, "signer-pre-register", [types.uint(amount_ustx)], sender.address);
  }

  // ---------------------------------------------------------------------------------
  // read only methods
  // ---------------------------------------------------------------------------------

  private callReadOnlyFn(method: string, args: Array<any> = [], sender: Account = this.deployer): ReadOnlyFn {
    const regex = /_/g;
    method = method.replace(regex, '-');
    const result = this.chain.callReadOnlyFn(this.name, method, args, sender?.address);
    return result;
  }

  is_authorized_as_stacker (stacker: string, burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("is_authorized_as_stacker", [types.principal(stacker), types.uint(burn_ht)]);
  }

  // Get the reward cycle for a given burn block height.
  // Runtime-panics if it's before first-burn-block-height.
  get_reward_cycle (burn_block_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("get_reward_cycle", [types.uint(burn_block_ht)]);
  }

  // Have all funds been disbursed from the last reward cycle?
  all_rewards_disbursed (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("all_rewards_disbursed", [types.uint(burn_ht)]);
  }

  // Tabulate how much BTC was won through PoX for the next 100 blocks.
  // Returns that tabulation over the next 100 blocks, as of the record stored in `disbursed_btc_state`
  // TODO: some benchmarking on this method will be needed to determine how much compute resources it uses.
  // It may need to be reduced (or increased!).
  get_next_btc_payout (rc: number, burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("get_next_btc_payout", [types.uint(burn_ht)]);
  }

  // The total BTC payout for the given reward cycle must already have been calculated.
  get_btc_owed (rc: number, signer: string): ReadOnlyFn {
    return this.callReadOnlyFn("get_next_btc_payout", [types.uint(rc), types.principal(signer)]);
  }

  // Verify that a Bitcoin transaction was mined on the Bitcoin chain
  // Returns (ok true) if so.
  // Returns (err ...) if not.
  // TODO: implement this
  authenticate_bitcoin_tx (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("authenticate_bitcoin_tx", [types.uint(burn_ht)]);
  }

  // Decode a raw Bitcoin transaction to extract its disbursement outputs.
  // Returns a list of up to 16 recipients and the BTC they each received.
  // TODO: implement this
  decode_disbursement_tx (tx: string, reward_cycle: number): ReadOnlyFn {
    return this.callReadOnlyFn("decode_disbursement_tx", [types.buff(tx), types.uint(reward_cycle)]);
  }

  // Is a burn block height in the registration window?
  in_registration_window (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("in_registration_window", [types.uint(burn_ht)]);
  }

  // Is a burn block height in the voting window?
  in_voting_window (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("in_voting_window", [types.uint(burn_ht)]);
  }

  // Is a burn block height in the transfer window?
  in_transfer_window (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("in_transfer_window", [types.uint(burn_ht)]);
  }

  // Is a burn block height in the penalty window?
  in_penalty_window (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("in_penalty_window", [types.uint(burn_ht)]);
  }

  // Can a stacker pre_register to be a signer?
  can_signer_pre_register (signer: string): ReadOnlyFn {
    return this.callReadOnlyFn("can_signer_pre_register", [types.principal(signer)]);
  }

  // Can a stacker register as a signer?
  can_signer_register (signer: string): ReadOnlyFn {
    return this.callReadOnlyFn("can_signer_register", [types.principal(signer)]);
  }

  // Can a Stacker vote for an address?
  can_signer_vote (signer: string): ReadOnlyFn {
    return this.callReadOnlyFn("can_signer_vote", [types.principal(signer)]);
  }

  // Get the winning PoX address for the upcoming reward cycle.
  // Only works if voting period for the upcoming reward cycle's sBTC wallet address is closed.
  inner_get_sbtc_wallet_addr (burn_ht: number): ReadOnlyFn {
    return this.callReadOnlyFn("inner_get_sbtc_wallet_addr", [types.uint(burn_ht)]);
  }

  // Get the sBTC wallet
  get_sbtc_wallet_addr(): ReadOnlyFn {
    return this.callReadOnlyFn("get_sbtc_wallet_addr", []);
  }

  // Determine how many signing slots a signer has for a given reward cycle
  // This determines how many shares of the signature this stacker must contribute.
  get_signing_slots (signer: string, reward_cycle: number): ReadOnlyFn {
    return this.callReadOnlyFn("get_signing_slots", [types.principal(signer), types.uint(reward_cycle)]);
  }

}
