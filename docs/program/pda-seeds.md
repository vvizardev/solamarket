# PDA Seeds

> Seed derivation for all program-owned accounts. On-chain and TypeScript derivations must match exactly.

---

## Seed Reference

| Account | Seeds |
|---------|-------|
| Market | `[b"market", question_hash: [u8;32]]` |
| Market vault authority | `[b"vault_authority", market_pubkey: [u8;32]]` |
| Order | `[b"order", market_pubkey: [u8;32], user_pubkey: [u8;32], nonce: [u8;8] (little-endian u64)]` |
| UserPosition | `[b"user_position", market_pubkey: [u8;32], user_pubkey: [u8;32]]` |

---

## Rust (On-Chain)

```rust
// utils/pda.rs

pub const SEED_MARKET:          &[u8] = b"market";
pub const SEED_VAULT_AUTHORITY: &[u8] = b"vault_authority";
pub const SEED_ORDER:           &[u8] = b"order";
pub const SEED_USER_POSITION:   &[u8] = b"user_position";

pub fn find_market_pda(question_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_MARKET, question_hash], program_id)
}

pub fn find_vault_authority_pda(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_VAULT_AUTHORITY, market.as_ref()], program_id)
}

pub fn find_order_pda(
    market: &Pubkey, user: &Pubkey, nonce: u64, program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_ORDER, market.as_ref(), user.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
}

pub fn find_user_position_pda(
    market: &Pubkey, user: &Pubkey, program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_USER_POSITION, market.as_ref(), user.as_ref()],
        program_id,
    )
}
```

---

## TypeScript (SDK)

```typescript
// sdk/src/pda.ts

const SEED_MARKET          = Buffer.from("market");
const SEED_VAULT_AUTHORITY = Buffer.from("vault_authority");
const SEED_ORDER           = Buffer.from("order");
const SEED_USER_POSITION   = Buffer.from("user_position");

export function findMarketPda(
  questionHash: Uint8Array,
  programId = PROGRAM_ID,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEED_MARKET, questionHash], programId);
}

export function findVaultAuthorityPda(
  marketPubkey: PublicKey,
  programId = PROGRAM_ID,
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
  programId = PROGRAM_ID,
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
  programId = PROGRAM_ID,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEED_USER_POSITION, marketPubkey.toBuffer(), userPubkey.toBuffer()],
    programId,
  );
}
```

---

## Critical: Nonce Encoding

The Order PDA includes `nonce` as an 8-byte **little-endian unsigned 64-bit integer**. This must match exactly between on-chain (`nonce.to_le_bytes()`) and TypeScript (`nonceBuf.writeBigUInt64LE(nonce)`).

A mismatch will produce a different PDA, causing the program to reject the `order_ai` account with `InvalidPda`.

---

## Bump Seeds

All PDA derivation functions return a `(Pubkey, u8)` tuple. The `u8` is the bump seed — the smallest nonce that makes the PDA off the ed25519 curve.

The bump is stored in each account struct (e.g., `Market.bump`, `Order.bump`) and used in `invoke_signed` calls:

```rust
// Signing on behalf of the vault_authority PDA
invoke_signed(
    &transfer_ix,
    accounts,
    &[&[SEED_VAULT_AUTHORITY, market.key.as_ref(), &[vault_auth_bump]]],
)?;
```

---

## Deriving Addresses Off-Chain

Any client that knows the program ID, market question, and user pubkey can derive all related account addresses without any RPC calls:

```typescript
import { createHash } from "crypto";
import {
  findMarketPda, findOrderPda, findUserPositionPda, findVaultAuthorityPda
} from "@polymarket-sol/sdk";

const question = "Will BTC be above $100k by end of 2025?";
const hash = new Uint8Array(createHash("sha256").update(question).digest());

const [marketPda]   = findMarketPda(hash, PROGRAM_ID);
const [vaultAuth]   = findVaultAuthorityPda(marketPda, PROGRAM_ID);
const [posPda]      = findUserPositionPda(marketPda, userPubkey, PROGRAM_ID);
const [orderPda]    = findOrderPda(marketPda, userPubkey, 1n, PROGRAM_ID);
```

---

## Next Steps

- [Account Structs](./accounts.md)
- [Instructions Reference](./instructions.md)
