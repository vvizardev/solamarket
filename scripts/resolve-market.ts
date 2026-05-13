/**
 * Resolve a market on-chain.
 *
 * Usage:
 *   PROGRAM_ID=<id> MARKET=<pubkey> OUTCOME=1  ts-node scripts/resolve-market.ts
 *   OUTCOME=1 → YES wins
 *   OUTCOME=2 → NO  wins
 */
import { PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { resolveMarketInstruction } from "@polymarket-sol/sdk";
import { getConnection, getProgramId, loadKeypair } from "./_common";

async function main() {
  const connection  = getConnection();
  const programId   = getProgramId();
  const admin       = loadKeypair("../wallet/admin.json");

  const marketStr = process.env.MARKET;
  const outcomeStr = process.env.OUTCOME;
  if (!marketStr) throw new Error("MARKET env var not set");
  if (!outcomeStr) throw new Error("OUTCOME env var not set (1=YES, 2=NO)");

  const marketPubkey = new PublicKey(marketStr);
  const outcome      = parseInt(outcomeStr, 10) as 1 | 2;
  if (outcome !== 1 && outcome !== 2) throw new Error("OUTCOME must be 1 (YES) or 2 (NO)");

  console.log(`Resolving market ${marketPubkey.toBase58().slice(0, 16)}…`);
  console.log(`  Outcome: ${outcome === 1 ? "YES" : "NO"}`);

  const ix = resolveMarketInstruction(admin.publicKey, marketPubkey, outcome, programId);
  const tx  = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(connection, tx, [admin]);

  console.log("  tx sig:", sig);
  console.log("Market resolved successfully.");
}

main().catch((e) => { console.error(e); process.exit(1); });
