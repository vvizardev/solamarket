import { Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import path from "path";

export interface KeeperConfig {
  rpcEndpoint:  string;
  wsEndpoint:   string;
  programId:    PublicKey;
  keeperKeypair: Keypair;
  markets:      PublicKey[];
  /** Milliseconds between fill-attempt retries when no cross exists. */
  pollInterval: number;
  /** Minimum fill size in collateral units (skip dust fills). */
  minFillSize:  bigint;
}

function loadKeypair(filePath: string): Keypair {
  const resolved = path.resolve(filePath);
  const raw      = JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

export function loadConfig(): KeeperConfig {
  const rpcEndpoint  = process.env.RPC_ENDPOINT  ?? "https://api.devnet.solana.com";
  const wsEndpoint   = process.env.WS_ENDPOINT   ?? "wss://api.devnet.solana.com";
  const programIdStr = process.env.PROGRAM_ID    ?? "11111111111111111111111111111111";
  const keypairPath  = process.env.KEEPER_KEYPAIR ?? "../../wallet/keeper.json";
  const marketsRaw   = process.env.MARKET_PUBKEYS ?? "";

  const markets = marketsRaw
    ? marketsRaw.split(",").map((s) => new PublicKey(s.trim()))
    : [];

  return {
    rpcEndpoint,
    wsEndpoint,
    programId:    new PublicKey(programIdStr),
    keeperKeypair: loadKeypair(keypairPath),
    markets,
    pollInterval:  parseInt(process.env.POLL_INTERVAL_MS ?? "2000", 10),
    minFillSize:   BigInt(process.env.MIN_FILL_SIZE ?? "1000"), // 0.001 USDC
  };
}
