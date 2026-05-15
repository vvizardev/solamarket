import * as dotenv from "dotenv";
import { PublicKey } from "@solana/web3.js";

dotenv.config();

export interface KeeperConfig {
  programId: PublicKey;
  keeperKeypairPath: string;
  marketPubkeys: PublicKey[];
  rpcEndpoint: string;
  wsEndpoint: string;
  pollIntervalMs: number;
  minFillSize: bigint;
}

export function loadConfig(): KeeperConfig {
  const programIdStr = process.env["PROGRAM_ID"];
  if (!programIdStr) throw new Error("Missing required env var: PROGRAM_ID");

  const marketPubkeysStr = process.env["MARKET_PUBKEYS"];
  if (!marketPubkeysStr) throw new Error("Missing required env var: MARKET_PUBKEYS");

  const marketPubkeys = marketPubkeysStr
    .split(",")
    .map(s => s.trim())
    .filter(s => s.length > 0)
    .map(s => new PublicKey(s));

  if (marketPubkeys.length === 0) {
    throw new Error("MARKET_PUBKEYS is set but contains no valid pubkeys");
  }

  return {
    programId: new PublicKey(programIdStr),
    keeperKeypairPath: process.env["KEEPER_KEYPAIR"] ?? "../../wallet/keeper.json",
    marketPubkeys,
    rpcEndpoint: process.env["RPC_ENDPOINT"] ?? "https://api.devnet.solana.com",
    wsEndpoint:  process.env["WS_ENDPOINT"]  ?? "wss://api.devnet.solana.com",
    pollIntervalMs: parseInt(process.env["POLL_INTERVAL_MS"] ?? "2000", 10),
    minFillSize: BigInt(process.env["MIN_FILL_SIZE"] ?? "1000"),
  };
}
