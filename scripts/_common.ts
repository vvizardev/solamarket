import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import path from "path";

export const RPC = process.env.RPC_ENDPOINT ?? "https://api.devnet.solana.com";

export function loadKeypair(relPath: string): Keypair {
  const resolved = path.resolve(__dirname, relPath);
  const raw      = JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

export function getConnection(): Connection {
  return new Connection(RPC, "confirmed");
}

export function getProgramId(): PublicKey {
  const id = process.env.PROGRAM_ID;
  if (!id) throw new Error("PROGRAM_ID env var not set");
  return new PublicKey(id);
}
