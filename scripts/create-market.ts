/**
 * Usage:
 *   PROGRAM_ID=<program_id> QUESTION="Will BTC > 100k?" END_TIME=1777000000 \
 *   ts-node scripts/create-market.ts
 */
import { Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  createMarketInstruction,
  findMarketPda,
  findVaultAuthorityPda,
  hashQuestion,
} from "@polymarket-sol/sdk";
import { getConnection, getProgramId, loadKeypair } from "./_common";

async function main() {
  const connection  = getConnection();
  const programId   = getProgramId();
  const admin       = loadKeypair("../wallet/admin.json");

  const question = process.env.QUESTION ?? "Will BTC exceed $100k by end of 2025?";
  const endTime  = BigInt(process.env.END_TIME ?? Math.floor(Date.now() / 1000) + 86400 * 30);

  const collateralMintStr = process.env.COLLATERAL_MINT;
  if (!collateralMintStr) {
    throw new Error("COLLATERAL_MINT env var not set. Run fund-wallet.ts first.");
  }
  const { PublicKey } = await import("@solana/web3.js");
  const collateralMint = new PublicKey(collateralMintStr);

  const questionHash     = await hashQuestion(question);
  const [marketPda]      = findMarketPda(questionHash, programId);
  const [vaultAuthority] = findVaultAuthorityPda(marketPda, programId);
  const vaultAta         = getAssociatedTokenAddressSync(collateralMint, vaultAuthority, true);

  console.log("Creating market...");
  console.log("  question:   ", question);
  console.log("  market PDA: ", marketPda.toBase58());
  console.log("  vault ATA:  ", vaultAta.toBase58());

  const ix = createMarketInstruction(
    admin.publicKey,
    marketPda,
    vaultAta,
    vaultAuthority,
    collateralMint,
    { questionHash, endTime },
    programId,
  );

  const tx  = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(connection, tx, [admin]);
  console.log("  tx sig:     ", sig);
  console.log("\nMarket created. Set MARKET_PUBKEYS=" + marketPda.toBase58());
}

main().catch((e) => { console.error(e); process.exit(1); });
