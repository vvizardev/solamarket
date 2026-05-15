import {
  AccountMeta,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { DLOBNode } from "./DLOB";
import { Market, FILL_ORDER_DISCRIMINANT } from "./types";
import { findUserPositionPda } from "./pda";

/**
 * Builds and submits `FillOrder` transactions (instruction discriminant 5).
 *
 * Account layout required by the on-chain program:
 *   0 — keeper          (writable, signer)
 *   1 — market PDA      (readonly)
 *   2 — bid_order PDA   (writable)
 *   3 — ask_order PDA   (writable)
 *   4 — bid UserPosition (writable)
 *   5 — ask UserPosition (writable)
 *   6 — keeper UserPosition (writable) — receives keeper_reward share
 *   7 — fee_recipient UserPosition (writable) — receives treasury_share + maker_fee
 *
 * Note: accounts #6 and #7 may be the same PDA when
 * `market.feeRecipientUser == keeper.publicKey`.
 */
export class Filler {
  private readonly connection: Connection;
  private readonly keeper:     Keypair;
  readonly programId:          PublicKey;
  private readonly minFillSize: bigint;

  constructor(
    connection:  Connection,
    keeper:      Keypair,
    programId:   PublicKey,
    minFillSize: bigint,
  ) {
    this.connection  = connection;
    this.keeper      = keeper;
    this.programId   = programId;
    this.minFillSize = minFillSize;
  }

  /**
   * Attempts to fill a crossing bid/ask pair.
   *
   * Returns the transaction signature on success, or null if the fill was
   * skipped (dust, simulation failure) or lost to another keeper.
   */
  async fill(
    bid:         DLOBNode,
    ask:         DLOBNode,
    marketPubkey: PublicKey,
    market:      Market,
  ): Promise<string | null> {
    const fillSize = bid.remaining < ask.remaining ? bid.remaining : ask.remaining;

    if (fillSize < this.minFillSize) {
      return null;
    }

    // Derive UserPosition PDAs for both sides and the keeper.
    const [bidUserPosPda]       = findUserPositionPda(marketPubkey, bid.order.user, this.programId);
    const [askUserPosPda]       = findUserPositionPda(marketPubkey, ask.order.user, this.programId);
    const [keeperPosPda]        = findUserPositionPda(marketPubkey, this.keeper.publicKey, this.programId);
    const [feeRecipientPosPda]  = findUserPositionPda(marketPubkey, market.feeRecipientUser, this.programId);

    // Instruction data: [discriminant u8] [fill_size u64 LE]
    const data = Buffer.alloc(9);
    data.writeUInt8(FILL_ORDER_DISCRIMINANT, 0);
    data.writeBigUInt64LE(fillSize, 1);

    const keys: AccountMeta[] = [
      { pubkey: this.keeper.publicKey, isSigner: true,  isWritable: true  }, // 0
      { pubkey: marketPubkey,          isSigner: false, isWritable: false }, // 1
      { pubkey: bid.pubkey,            isSigner: false, isWritable: true  }, // 2
      { pubkey: ask.pubkey,            isSigner: false, isWritable: true  }, // 3
      { pubkey: bidUserPosPda,         isSigner: false, isWritable: true  }, // 4
      { pubkey: askUserPosPda,         isSigner: false, isWritable: true  }, // 5
      { pubkey: keeperPosPda,          isSigner: false, isWritable: true  }, // 6
      { pubkey: feeRecipientPosPda,    isSigner: false, isWritable: true  }, // 7
    ];

    const instruction = new TransactionInstruction({
      programId: this.programId,
      keys,
      data,
    });

    const tx = new Transaction().add(instruction);
    tx.feePayer = this.keeper.publicKey;

    // ── Simulate before spending SOL ─────────────────────────────────────────
    try {
      const { blockhash } = await this.connection.getLatestBlockhash("confirmed");
      tx.recentBlockhash = blockhash;

      const sim = await this.connection.simulateTransaction(tx, [this.keeper]);
      if (sim.value.err) {
        console.warn(
          `[Filler] simulation error for bid=${bid.pubkey.toBase58().slice(0, 8)} ask=${ask.pubkey.toBase58().slice(0, 8)}:`,
          sim.value.err,
        );
        return null;
      }
    } catch (err) {
      console.warn("[Filler] simulation threw:", err instanceof Error ? err.message : err);
      return null;
    }

    // ── Submit ───────────────────────────────────────────────────────────────
    try {
      const sig = await sendAndConfirmTransaction(this.connection, tx, [this.keeper], {
        commitment: "confirmed",
      });
      console.log(`[Filler] filled ${fillSize} — tx: ${sig}`);
      bid.applyFill(fillSize);
      ask.applyFill(fillSize);
      return sig;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.includes("AccountNotFound") || msg.includes("custom program error: 0x19")) {
        // Order PDA already closed — another keeper won the race
        console.info("[Filler] order already filled by another keeper");
      } else {
        console.error("[Filler] fill failed:", msg);
      }
      return null;
    }
  }
}
