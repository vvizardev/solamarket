import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import {
  DLOBNode,
  fillOrderInstruction,
  findUserPositionPda,
} from "@polymarket-sol/sdk";

export interface FillerOptions {
  connection:    Connection;
  keeper:        Keypair;
  programId:     PublicKey;
  minFillSize:   bigint;
}

/**
 * Constructs, simulates, and submits FillOrder transactions.
 * Permissionless — any keypair can be the keeper.
 */
export class Filler {
  private connection:  Connection;
  private keeper:      Keypair;
  private programId:   PublicKey;
  private minFillSize: bigint;
  private keeperPositionCache = new Map<string, PublicKey>();

  constructor(opts: FillerOptions) {
    this.connection  = opts.connection;
    this.keeper      = opts.keeper;
    this.programId   = opts.programId;
    this.minFillSize = opts.minFillSize;
  }

  async fill(
    marketPubkey: PublicKey,
    bid: DLOBNode,
    ask: DLOBNode,
  ): Promise<string | null> {
    const fillSize = bid.remaining < ask.remaining ? bid.remaining : ask.remaining;
    if (fillSize < this.minFillSize) {
      return null; // skip dust
    }

    const [bidPosPda]    = findUserPositionPda(marketPubkey, bid.order.user, this.programId);
    const [askPosPda]    = findUserPositionPda(marketPubkey, ask.order.user, this.programId);
    const keeperPosPda   = this.getOrDeriveKeeperPosition(marketPubkey);

    const ix = fillOrderInstruction(
      this.keeper.publicKey,
      marketPubkey,
      bid.pubkey,
      ask.pubkey,
      bidPosPda,
      askPosPda,
      keeperPosPda,
      { fillSize },
      this.programId,
    );

    const tx = new Transaction().add(ix);

    // Simulate first — skip if it would fail (race condition: order already filled)
    const simulation = await this.connection.simulateTransaction(tx, [this.keeper]);
    if (simulation.value.err) {
      console.warn(
        `[Filler] simulation error for bid=${bid.pubkey.toBase58().slice(0, 8)} ask=${ask.pubkey.toBase58().slice(0, 8)}:`,
        simulation.value.err,
      );
      return null;
    }

    try {
      const sig = await sendAndConfirmTransaction(this.connection, tx, [this.keeper], {
        commitment: "confirmed",
      });
      console.log(`[Filler] filled ${fillSize} — tx: ${sig}`);
      bid.applyFill(fillSize);
      ask.applyFill(fillSize);
      return sig;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.includes("AccountNotFound") || msg.includes("custom program error: 0x19")) {
        // Order no longer exists; already taken by another keeper
        console.info("[Filler] order already filled by another keeper");
      } else {
        console.error("[Filler] fill failed:", msg);
      }
      return null;
    }
  }

  private getOrDeriveKeeperPosition(marketPubkey: PublicKey): PublicKey {
    const key = marketPubkey.toBase58();
    if (!this.keeperPositionCache.has(key)) {
      const [pda] = findUserPositionPda(marketPubkey, this.keeper.publicKey, this.programId);
      this.keeperPositionCache.set(key, pda);
    }
    return this.keeperPositionCache.get(key)!;
  }
}
