import { Connection, PublicKey } from "@solana/web3.js";
import { OrderSubscriber, DLOB, DLOBNode } from "@polymarket-sol/sdk";
import { loadConfig } from "./config";
import { Filler } from "./Filler";

async function runMarket(
  connection:  Connection,
  marketPubkey: PublicKey,
  filler:      Filler,
  programId:   PublicKey,
  pollInterval: number,
): Promise<void> {
  const subscriber = new OrderSubscriber(connection, marketPubkey, programId);

  subscriber.onUpdate((_pubkey, _node) => {
    // Eagerly try to fill on any order change
    tryCross(subscriber.getDLOB(), marketPubkey, filler).catch(console.error);
  });

  await subscriber.subscribe();
  console.log(
    `[Keeper] subscribed to market ${marketPubkey.toBase58().slice(0, 8)}… ` +
    `bids=${subscriber.getDLOB().bidCount} asks=${subscriber.getDLOB().askCount}`,
  );

  // Also poll periodically (backup if WebSocket misses an event)
  setInterval(() => {
    tryCross(subscriber.getDLOB(), marketPubkey, filler).catch(console.error);
  }, pollInterval);
}

async function tryCross(
  dlob:         DLOB,
  marketPubkey: PublicKey,
  filler:       Filler,
): Promise<void> {
  const cross = dlob.findCross();
  if (!cross) return;

  const [bid, ask] = cross;
  await filler.fill(marketPubkey, bid, ask);
}

async function main(): Promise<void> {
  const cfg = loadConfig();

  const connection = new Connection(cfg.rpcEndpoint, {
    wsEndpoint:  cfg.wsEndpoint,
    commitment:  "confirmed",
  });

  const filler = new Filler({
    connection,
    keeper:      cfg.keeperKeypair,
    programId:   cfg.programId,
    minFillSize: cfg.minFillSize,
  });

  console.log(
    `[Keeper] starting — program=${cfg.programId.toBase58().slice(0, 8)} ` +
    `keeper=${cfg.keeperKeypair.publicKey.toBase58().slice(0, 8)} ` +
    `markets=${cfg.markets.length}`,
  );

  if (cfg.markets.length === 0) {
    console.warn("[Keeper] no markets configured — set MARKET_PUBKEYS env var");
  }

  await Promise.all(
    cfg.markets.map((market) =>
      runMarket(connection, market, filler, cfg.programId, cfg.pollInterval),
    ),
  );

  // Keep the process alive
  await new Promise<never>(() => {});
}

main().catch((err) => {
  console.error("[Keeper] fatal:", err);
  process.exit(1);
});
