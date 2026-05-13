import {
  Connection,
  PublicKey,
  KeyedAccountInfo,
  Context,
} from "@solana/web3.js";
import { deserializeOrder } from "../accounts";
import { PROGRAM_ID } from "../constants";
import { DLOB } from "./DLOB";
import { DLOBNode } from "./DLOBNode";

export type OrderUpdateCallback = (
  pubkey: PublicKey,
  node:   DLOBNode | null,   // null means the account was deleted
) => void;

/**
 * Subscribes to on-chain Order account changes for a single market.
 *
 * Uses `getProgramAccounts` for initial population, then `onProgramAccountChange`
 * (WebSocket) for incremental updates.
 */
export class OrderSubscriber {
  private connection:   Connection;
  private marketPubkey: PublicKey;
  private programId:    PublicKey;
  private dlob:         DLOB;
  private subscriptionId: number | undefined;
  private callbacks:    OrderUpdateCallback[] = [];

  constructor(
    connection:   Connection,
    marketPubkey: PublicKey,
    programId:    PublicKey = PROGRAM_ID,
  ) {
    this.connection   = connection;
    this.marketPubkey = marketPubkey;
    this.programId    = programId;
    this.dlob         = new DLOB();
  }

  // ── public API ────────────────────────────────────────────────────────────

  onUpdate(cb: OrderUpdateCallback): void {
    this.callbacks.push(cb);
  }

  getDLOB(): DLOB {
    return this.dlob;
  }

  /**
   * 1. Fetches all existing Order accounts (initial snapshot).
   * 2. Opens a WebSocket subscription for incremental updates.
   */
  async subscribe(): Promise<void> {
    await this.loadInitialSnapshot();
    this.openWebSocket();
  }

  unsubscribe(): void {
    if (this.subscriptionId !== undefined) {
      this.connection.removeProgramAccountChangeListener(this.subscriptionId);
      this.subscriptionId = undefined;
    }
  }

  // ── private ───────────────────────────────────────────────────────────────

  private async loadInitialSnapshot(): Promise<void> {
    const accounts = await this.connection.getProgramAccounts(this.programId, {
      filters: [
        // discriminant byte = 1 (Order)
        {
          memcmp: {
            offset: 0,
            bytes:  Buffer.from([1]).toString("base64"),
          },
        },
        // market pubkey at offset 1
        {
          memcmp: {
            offset: 1,
            bytes:  this.marketPubkey.toBase58(),
          },
        },
      ],
    });

    for (const { pubkey, account } of accounts) {
      try {
        const order = deserializeOrder(Buffer.from(account.data));
        const node  = new DLOBNode(pubkey, order);
        this.dlob.insert(node);
      } catch {
        // malformed account — skip
      }
    }
  }

  private openWebSocket(): void {
    this.subscriptionId = this.connection.onProgramAccountChange(
      this.programId,
      (info: KeyedAccountInfo, _ctx: Context) => {
        const { accountId, accountInfo } = info;
        this.handleAccountChange(accountId, Buffer.from(accountInfo.data));
      },
      "confirmed",
      [
        {
          memcmp: {
            offset: 0,
            bytes:  Buffer.from([1]).toString("base64"),
          },
        },
        {
          memcmp: {
            offset: 1,
            bytes:  this.marketPubkey.toBase58(),
          },
        },
      ],
    );
  }

  private handleAccountChange(pubkey: PublicKey, data: Buffer): void {
    if (data.length === 0) {
      // Account closed (fully filled or cancelled)
      this.dlob.remove(pubkey);
      this.emit(pubkey, null);
      return;
    }

    try {
      const order = deserializeOrder(data);
      const node  = new DLOBNode(pubkey, order);
      this.dlob.update(pubkey, node);
      this.emit(pubkey, node);
    } catch {
      // ignore malformed
    }
  }

  private emit(pubkey: PublicKey, node: DLOBNode | null): void {
    for (const cb of this.callbacks) {
      cb(pubkey, node);
    }
  }
}
