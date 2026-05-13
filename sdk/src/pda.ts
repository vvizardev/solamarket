import { PublicKey } from "@solana/web3.js";
import { PROGRAM_ID } from "./constants";

// ── seed constants (mirror on-chain utils/pda.rs) ──────────────────────────

const SEED_MARKET          = Buffer.from("market");
const SEED_VAULT_AUTHORITY = Buffer.from("vault_authority");
const SEED_ORDER           = Buffer.from("order");
const SEED_USER_POSITION   = Buffer.from("user_position");

// ── derivation helpers ─────────────────────────────────────────────────────

export function findMarketPda(
  questionHash: Uint8Array,
  programId: PublicKey = PROGRAM_ID,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_MARKET, questionHash],
    programId,
  );
}

export function findVaultAuthorityPda(
  marketPubkey: PublicKey,
  programId: PublicKey = PROGRAM_ID,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_VAULT_AUTHORITY, marketPubkey.toBuffer()],
    programId,
  );
}

export function findOrderPda(
  marketPubkey: PublicKey,
  userPubkey:   PublicKey,
  nonce:        bigint,
  programId:    PublicKey = PROGRAM_ID,
): [PublicKey, number] {
  const nonceBuf = Buffer.alloc(8);
  nonceBuf.writeBigUInt64LE(nonce);
  return PublicKey.findProgramAddressSync(
    [SEED_ORDER, marketPubkey.toBuffer(), userPubkey.toBuffer(), nonceBuf],
    programId,
  );
}

export function findUserPositionPda(
  marketPubkey: PublicKey,
  userPubkey:   PublicKey,
  programId:    PublicKey = PROGRAM_ID,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_USER_POSITION, marketPubkey.toBuffer(), userPubkey.toBuffer()],
    programId,
  );
}
