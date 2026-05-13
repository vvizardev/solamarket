/**
 * Creates a mock USDC mint on devnet and mints tokens to the admin and an
 * optional recipient wallet.
 *
 * Usage:
 *   ts-node scripts/fund-wallet.ts
 */
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { getConnection, loadKeypair } from "./_common";

async function main() {
  const connection = getConnection();
  const admin      = loadKeypair("../wallet/admin.json");

  // Create a mock USDC mint (6 decimals, admin is both mint authority and freeze authority)
  console.log("Creating mock USDC mint...");
  const mint = await createMint(
    connection,
    admin,
    admin.publicKey,  // mint authority
    admin.publicKey,  // freeze authority
    6,                // decimals (matches real USDC)
  );
  console.log("  Mint address:", mint.toBase58());

  // Create ATA for admin and mint 10 000 USDC
  const adminAta = await getOrCreateAssociatedTokenAccount(
    connection,
    admin,
    mint,
    admin.publicKey,
  );
  await mintTo(connection, admin, mint, adminAta.address, admin, 10_000 * 1_000_000);
  console.log("  Minted 10 000 USDC to admin ATA:", adminAta.address.toBase58());

  // Optionally fund the keeper wallet too
  try {
    const keeper    = loadKeypair("../wallet/keeper.json");
    const keeperAta = await getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      mint,
      keeper.publicKey,
    );
    await mintTo(connection, admin, mint, keeperAta.address, admin, 1_000 * 1_000_000);
    console.log("  Minted 1 000 USDC to keeper ATA:", keeperAta.address.toBase58());
  } catch {
    console.log("  No keeper.json found — skipping keeper USDC airdrop.");
  }

  console.log("\nSet the following env vars:");
  console.log(`  COLLATERAL_MINT=${mint.toBase58()}`);
}

main().catch((e) => { console.error(e); process.exit(1); });
