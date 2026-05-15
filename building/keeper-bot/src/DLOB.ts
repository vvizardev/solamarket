import { PublicKey } from "@solana/web3.js";
import { Order } from "./types";

/**
 * Wraps a single on-chain Order with helpers used by the keeper.
 */
export class DLOBNode {
  readonly pubkey: PublicKey;
  readonly order: Order;

  constructor(pubkey: PublicKey, order: Order) {
    this.pubkey = pubkey;
    this.order  = order;
  }

  /** Unfilled collateral units remaining in this order. */
  get remaining(): bigint {
    return this.order.size - this.order.fillAmount;
  }

  /** Optimistically update local fill state after a successful `FillOrder`. */
  applyFill(amount: bigint): void {
    this.order.fillAmount += amount;
  }
}

/**
 * Decentralised Limit Order Book (DLOB) — in-memory sorted order book.
 *
 * Bids are sorted descending by price (highest bid fills first).
 * Asks are sorted ascending  by price (lowest ask fills first).
 *
 * The book is keyed by order account pubkey so updates and removals are O(1).
 * Cross detection is O(n) in the worst case but markets will be small enough
 * that this is negligible compared to RPC latency.
 */
export class DLOB {
  private readonly bids = new Map<string, DLOBNode>();
  private readonly asks = new Map<string, DLOBNode>();

  /** Insert or replace an order in the book. */
  upsert(pubkey: PublicKey, order: Order): DLOBNode {
    const key  = pubkey.toBase58();
    const node = new DLOBNode(pubkey, order);
    if (order.side === 0) {
      this.bids.set(key, node);
    } else {
      this.asks.set(key, node);
    }
    return node;
  }

  /** Remove an order from the book (e.g. fully filled or cancelled). */
  remove(pubkey: PublicKey): void {
    const key = pubkey.toBase58();
    this.bids.delete(key);
    this.asks.delete(key);
  }

  /** Best (highest-priced) bid with remaining > 0, or null. */
  bestBid(): DLOBNode | null {
    let best: DLOBNode | null = null;
    for (const node of this.bids.values()) {
      if (node.remaining <= 0n) continue;
      if (!best || node.order.price > best.order.price) best = node;
    }
    return best;
  }

  /** Best (lowest-priced) ask with remaining > 0, or null. */
  bestAsk(): DLOBNode | null {
    let best: DLOBNode | null = null;
    for (const node of this.asks.values()) {
      if (node.remaining <= 0n) continue;
      if (!best || node.order.price < best.order.price) best = node;
    }
    return best;
  }

  /**
   * Returns [bid, ask] when `best_bid.price >= best_ask.price`, otherwise null.
   * This is the standard DLOB crossing condition.
   */
  findCross(): [DLOBNode, DLOBNode] | null {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return null;
    if (bid.order.price >= ask.order.price) return [bid, ask];
    return null;
  }

  /** Number of bid entries (including fully filled). */
  get bidCount(): number {
    return this.bids.size;
  }

  /** Number of ask entries (including fully filled). */
  get askCount(): number {
    return this.asks.size;
  }

  /** All bid nodes with remaining > 0, sorted descending by price. */
  activeBids(): DLOBNode[] {
    return Array.from(this.bids.values())
      .filter(n => n.remaining > 0n)
      .sort((a, b) => (b.order.price > a.order.price ? 1 : -1));
  }

  /** All ask nodes with remaining > 0, sorted ascending by price. */
  activeAsks(): DLOBNode[] {
    return Array.from(this.asks.values())
      .filter(n => n.remaining > 0n)
      .sort((a, b) => (a.order.price > b.order.price ? 1 : -1));
  }
}
