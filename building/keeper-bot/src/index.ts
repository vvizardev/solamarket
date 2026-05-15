import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import { loadConfig } from "./config";
import { deserializeMarket } from "./deserialize";
import { DLOB } from "./DLOB";
import { Filler } from "./Filler";
import { Market } from "./types";
import { OrderSubscriber } from "./OrderSubscriber";

// ── helpers ───────────────────────────────────────────────────────────────────

async function fetchMarket(
  connection: Connection,
  marketPubkey: PublicKey,
): Promise<Market> {
  const info = await connection.getAccountInfo(marketPubkey, "confirmed");
  if (!info) throw new Error(`Market account not found: ${marketPubkey.toBase58()}`);
  return deserializeMarket(Buffer.from(info.data));
}

async function tryCross(
  dlob: DLOB,
  filler: Filler,
  marketPubkey: PublicKey,
  market: Market,
): Promise<void> {
  const cross = dlob.findCross();
  if (!cross) return;
  const [bid, ask] = cross;
  await filler.fill(bid, ask, marketPubkey, market);
}

// ── per-market worker ─────────────────────────────────────────────────────────

async function runMarket(
  connection:    Connection,
  marketPubkey:  PublicKey,
  filler:        Filler,
  pollIntervalMs: number,
): Promise<void> {
  const short = marketPubkey.toBase58().slice(0, 8);

  const market = await fetchMarket(connection, marketPubkey);

  if (market.resolved) {
    console.log(`[Keeper] market ${short}… is already resolved — skipping`);
    return;
  }

  const subscriber = new OrderSubscriber(connection, marketPubkey, filler.programId);

  // React to every DLOB change — try to fill crosses immediately.
  subscriber.onUpdate((_pubkey, _node) => {
    tryCross(subscriber.getDLOB(), filler, marketPubkey, market).catch(err =>
      console.error(`[Keeper] tryCross error (market=${short}):`, err),
    );
  });

  // Subscribe: initial snapshot + WebSocket + poll fallback.
  await subscriber.subscribe(pollIntervalMs);

  const dlob = subscriber.getDLOB();
  console.log(
    `[Keeper] subscribed to market ${short}… bids=${dlob.bidCount} asks=${dlob.askCount}`,
  );

  // Check for any crosses that were already in the snapshot.
  await tryCross(dlob, filler, marketPubkey, market);

  // Secondary interval — ensures we re-check even when WebSocket is quiet.
  setInterval(() => {
    tryCross(subscriber.getDLOB(), filler, marketPubkey, market).catch(err =>
      console.error(`[Keeper] interval tryCross error (market=${short}):`, err),
    );
  }, pollIntervalMs);
}

// ── entry point ───────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  const config = loadConfig();

  const keypairPath = path.resolve(config.keeperKeypairPath);
  if (!fs.existsSync(keypairPath)) {
    console.error(`[Keeper] keypair file not found: ${keypairPath}`);
    process.exit(1);
  }
  const rawKeypair = JSON.parse(fs.readFileSync(keypairPath, "utf-8")) as number[];
  const keeperKeypair = Keypair.fromSecretKey(new Uint8Array(rawKeypair));

  const connection = new Connection(config.rpcEndpoint, {
    wsEndpoint:  config.wsEndpoint,
    commitment:  "confirmed",
  });

  const filler = new Filler(connection, keeperKeypair, config.programId, config.minFillSize);

  console.log(
    `[Keeper] starting — program=${config.programId.toBase58().slice(0, 8)} ` +
    `keeper=${keeperKeypair.publicKey.toBase58().slice(0, 8)} ` +
    `markets=${config.marketPubkeys.length}`,
  );

  // Run each market concurrently.  Errors in one market do not kill others.
  await Promise.allSettled(
    config.marketPubkeys.map(pk =>
      runMarket(connection, pk, filler, config.pollIntervalMs).catch(err =>
        console.error(`[Keeper] fatal error for market ${pk.toBase58().slice(0, 8)}:`, err),
      ),
    ),
  );

  // Keep the process alive (subscriptions and timers hold the event loop open).
  await new Promise<never>(() => { /* run forever */ });
}

main().catch(err => {
  console.error("[Keeper] startup failed:", err);
  process.exit(1);
});
