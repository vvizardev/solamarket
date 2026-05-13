import { PublicKey } from "@solana/web3.js";
import { OrderSide } from "../types";
import { DLOBNode } from "./DLOBNode";

/**
 * In-memory Decentralised Limit Order Book for a single market.
 *
 * Bids: sorted price DESC, then insertion time ASC (FIFO).
 * Asks: sorted price ASC,  then insertion time ASC (FIFO).
 */
export class DLOB {
  private bids = new Map<string, DLOBNode>();
  private asks = new Map<string, DLOBNode>();

  // ── mutations ────────────────────────────────────────────────────────────

  insert(node: DLOBNode): void {
    if (node.isFullyFilled) return;
    const key = node.pubkey.toBase58();
    if (node.side === OrderSide.Bid) {
      this.bids.set(key, node);
    } else {
      this.asks.set(key, node);
    }
  }

  update(pubkey: PublicKey, updatedNode: DLOBNode): void {
    const key = pubkey.toBase58();
    if (updatedNode.isFullyFilled) {
      this.bids.delete(key);
      this.asks.delete(key);
    } else if (updatedNode.side === OrderSide.Bid) {
      this.bids.set(key, updatedNode);
    } else {
      this.asks.set(key, updatedNode);
    }
  }

  remove(pubkey: PublicKey): void {
    const key = pubkey.toBase58();
    this.bids.delete(key);
    this.asks.delete(key);
  }

  // ── queries ──────────────────────────────────────────────────────────────

  /** Best bid (highest price). */
  bestBid(): DLOBNode | undefined {
    return this.sortedBids()[0];
  }

  /** Best ask (lowest price). */
  bestAsk(): DLOBNode | undefined {
    return this.sortedAsks()[0];
  }

  /** Returns bids sorted price DESC, FIFO within the same price. */
  sortedBids(): DLOBNode[] {
    return [...this.bids.values()].sort((a, b) => {
      if (b.price !== a.price) return b.price > a.price ? 1 : -1;
      return Number(a.order.createdAt - b.order.createdAt);
    });
  }

  /** Returns asks sorted price ASC, FIFO within the same price. */
  sortedAsks(): DLOBNode[] {
    return [...this.asks.values()].sort((a, b) => {
      if (a.price !== b.price) return a.price < b.price ? -1 : 1;
      return Number(a.order.createdAt - b.order.createdAt);
    });
  }

  /**
   * Find the first crossing pair.
   * Returns [bid, ask] if best_bid.price >= best_ask.price, else undefined.
   */
  findCross(): [DLOBNode, DLOBNode] | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return undefined;
    if (bid.price >= ask.price) return [bid, ask];
    return undefined;
  }

  get bidCount(): number {
    return this.bids.size;
  }

  get askCount(): number {
    return this.asks.size;
  }
}
