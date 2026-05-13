import { PublicKey } from "@solana/web3.js";
import { Order, OrderSide } from "../types";

/**
 * A single node in the DLOB — wraps an on-chain Order account.
 * Maintains a local fill-amount cache so the keeper can track partial fills
 * without waiting for the next RPC update.
 */
export class DLOBNode {
  readonly pubkey:    PublicKey;
  readonly order:     Order;
  /** Cached fill amount — may lead local state; reconciled on account updates. */
  private _cachedFill: bigint;

  constructor(pubkey: PublicKey, order: Order) {
    this.pubkey      = pubkey;
    this.order       = order;
    this._cachedFill = order.fillAmount;
  }

  get side(): OrderSide {
    return this.order.side as OrderSide;
  }

  get price(): bigint {
    return this.order.price;
  }

  get remaining(): bigint {
    return this.order.size - this._cachedFill;
  }

  get isFullyFilled(): boolean {
    return this._cachedFill >= this.order.size;
  }

  /** Update local cache after a successful FillOrder CPI. */
  applyFill(amount: bigint): void {
    this._cachedFill += amount;
    if (this._cachedFill > this.order.size) {
      this._cachedFill = this.order.size;
    }
  }

  /** Reconcile with freshly-fetched on-chain data. */
  reconcile(onChainFill: bigint): void {
    if (onChainFill > this._cachedFill) {
      this._cachedFill = onChainFill;
    }
  }
}
