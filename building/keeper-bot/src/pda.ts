import { PublicKey } from "@solana/web3.js";

const SEED_MARKET          = Buffer.from("market");
const SEED_VAULT_AUTHORITY = Buffer.from("vault_authority");
const SEED_ORDER           = Buffer.from("order");
const SEED_USER_POSITION   = Buffer.from("user_position");
const SEED_EVENT           = Buffer.from("event");

/**
 * Market PDA — seeds: ["market", question_hash]
 */
export function findMarketPda(
  questionHash: Uint8Array,
  programId: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEED_MARKET, questionHash], programId);
}

/**
 * Vault authority PDA — seeds: ["vault_authority", market_pubkey]
 */
export function findVaultAuthorityPda(
  marketPubkey: PublicKey,
  programId: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_VAULT_AUTHORITY, marketPubkey.toBuffer()],
    programId,
  );
}

/**
 * Order PDA — seeds: ["order", market_pubkey, user_pubkey, nonce (8-byte LE)]
 *
 * CRITICAL: nonce must be encoded as little-endian u64 to match Rust's
 * `nonce.to_le_bytes()`. Any mismatch produces a different PDA.
 */
export function findOrderPda(
  marketPubkey: PublicKey,
  userPubkey: PublicKey,
  nonce: bigint,
  programId: PublicKey,
): [PublicKey, number] {
  const nonceBuf = Buffer.alloc(8);
  nonceBuf.writeBigUInt64LE(nonce);
  return PublicKey.findProgramAddressSync(
    [SEED_ORDER, marketPubkey.toBuffer(), userPubkey.toBuffer(), nonceBuf],
    programId,
  );
}

/**
 * UserPosition PDA — seeds: ["user_position", market_pubkey, user_pubkey]
 */
export function findUserPositionPda(
  marketPubkey: PublicKey,
  userPubkey: PublicKey,
  programId: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_USER_POSITION, marketPubkey.toBuffer(), userPubkey.toBuffer()],
    programId,
  );
}

/**
 * Event PDA — seeds: ["event", event_id]
 */
export function findEventPda(
  eventId: Uint8Array,
  programId: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEED_EVENT, eventId], programId);
}
