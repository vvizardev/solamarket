import {
  Connection,
  KeyedAccountInfo,
  PublicKey,
} from "@solana/web3.js";
import bs58 from "bs58";
import { DLOB, DLOBNode } from "./DLOB";
import { deserializeOrder } from "./deserialize";
import { DISCRIMINANT_ORDER } from "./types";

export type UpdateCallback = (pubkey: PublicKey, node: DLOBNode | null) => void;

/**
 * Subscribes to all Order accounts for a single market.
 *
 * On `subscribe()`:
 *   1. Fetches a full snapshot via `getProgramAccounts`.
 *   2. Opens a `programSubscribe` WebSocket filtered to Order accounts for
 *      this market so new orders and updates are delivered in real time.
 *   3. Optionally polls `getProgramAccounts` on a timer as a fallback for
 *      missed WebSocket events or account deletions.
 */
export class OrderSubscriber {
  private readonly connection:   Connection;
  private readonly marketPubkey: PublicKey;
  private readonly programId:    PublicKey;
  private readonly dlob:         DLOB = new DLOB();
  private readonly callbacks:    UpdateCallback[] = [];

  private wsSubscriptionId: number | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;

  constructor(connection: Connection, marketPubkey: PublicKey, programId: PublicKey) {
    this.connection   = connection;
    this.marketPubkey = marketPubkey;
    this.programId    = programId;
  }

  /**
   * Load initial snapshot + start WebSocket subscription.
   * @param pollIntervalMs - if > 0, also poll as a fallback (default: no poll)
   */
  async subscribe(pollIntervalMs = 0): Promise<void> {
    await this.loadSnapshot();

    // Solana `programSubscribe` — filters are memcmp so we only receive
    // events for Order accounts belonging to this market.
    this.wsSubscriptionId = this.connection.onProgramAccountChange(
      this.programId,
      (keyedInfo: KeyedAccountInfo) => {
        this.handleAccountChange(keyedInfo.accountId, Buffer.from(keyedInfo.accountInfo.data));
      },
      "confirmed",
      [
        // discriminant byte = 1 (Order)
        { memcmp: { offset: 0, bytes: bs58.encode(Buffer.from([DISCRIMINANT_ORDER])) } },
        // market field at offset 1
        { memcmp: { offset: 1, bytes: this.marketPubkey.toBase58() } },
      ],
    );

    if (pollIntervalMs > 0) {
      this.pollTimer = setInterval(() => {
        this.loadSnapshot().catch(err =>
          console.error("[OrderSubscriber] poll snapshot failed:", err),
        );
      }, pollIntervalMs);
    }
  }

  /** Stop all subscriptions and timers. */
  async unsubscribe(): Promise<void> {
    if (this.pollTimer !== null) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
    if (this.wsSubscriptionId !== null) {
      await this.connection.removeProgramAccountChangeListener(this.wsSubscriptionId);
      this.wsSubscriptionId = null;
    }
  }

  getDLOB(): DLOB {
    return this.dlob;
  }

  /** Register a callback invoked on every DLOB change.
   *  `node === null` means the account was closed or the order is fully filled. */
  onUpdate(callback: UpdateCallback): void {
    this.callbacks.push(callback);
  }

  // ── private ────────────────────────────────────────────────────────────────

  private handleAccountChange(pubkey: PublicKey, data: Buffer): void {
    if (data.length === 0) {
      // Account deleted (closed on full fill or cancel)
      this.dlob.remove(pubkey);
      this.emit(pubkey, null);
      return;
    }

    // Guard: only process Order accounts (discriminant check)
    if (data[0] !== DISCRIMINANT_ORDER) return;

    try {
      const order = deserializeOrder(data);
      // Ignore orders for other markets that slipped past the filter
      if (!order.market.equals(this.marketPubkey)) return;

      if (order.size - order.fillAmount <= 0n) {
        // Fully filled — treat as closed
        this.dlob.remove(pubkey);
        this.emit(pubkey, null);
      } else {
        const node = this.dlob.upsert(pubkey, order);
        this.emit(pubkey, node);
      }
    } catch {
      // Not a valid Order account — skip silently
    }
  }

  private async loadSnapshot(): Promise<void> {
    const accounts = await this.connection.getProgramAccounts(this.programId, {
      filters: [
        { memcmp: { offset: 0, bytes: bs58.encode(Buffer.from([DISCRIMINANT_ORDER])) } },
        { memcmp: { offset: 1, bytes: this.marketPubkey.toBase58() } },
      ],
    });

    for (const { pubkey, account } of accounts) {
      const data = Buffer.from(account.data);
      try {
        const order = deserializeOrder(data);
        if (order.size - order.fillAmount > 0n) {
          this.dlob.upsert(pubkey, order);
        } else {
          this.dlob.remove(pubkey);
        }
      } catch {
        // Skip malformed accounts
      }
    }

    // Remove any entries no longer returned by getProgramAccounts
    // (closed accounts that the WebSocket may have missed).
    const liveKeys = new Set(accounts.map(a => a.pubkey.toBase58()));
    for (const node of [...this.dlob.activeBids(), ...this.dlob.activeAsks()]) {
      if (!liveKeys.has(node.pubkey.toBase58())) {
        this.dlob.remove(node.pubkey);
        this.emit(node.pubkey, null);
      }
    }
  }

  private emit(pubkey: PublicKey, node: DLOBNode | null): void {
    for (const cb of this.callbacks) {
      try {
        cb(pubkey, node);
      } catch (err) {
        console.error("[OrderSubscriber] callback threw:", err);
      }
    }
  }
}
