# Markets

> Binary markets, question hashes, lifecycle states, and admin responsibilities.

---

## What Is a Market?

A market is a binary prediction question with exactly two outcomes: **YES** and **NO**. Each outcome token is worth **1 USDC** if its outcome wins, and **0 USDC** if it loses. Prices are therefore natural probabilities — a YES price of 0.60 (6000 basis points) implies a 60% chance the question resolves YES.

Every market is stored in a single `Market` PDA on-chain. There is one market per question.

---

## Market Account Fields

```rust
pub struct Market {
    pub discriminant:     u8,       // = 0 (used for getProgramAccounts filter)
    pub question_hash:    [u8; 32], // SHA-256(question string)
    pub vault:            Pubkey,   // USDC ATA controlled by vault_authority PDA
    pub collateral_mint:  Pubkey,   // mock USDC mint (devnet)
    pub yes_mint:         Pubkey,   // default() until TokenizePosition is called
    pub no_mint:          Pubkey,   // default() until TokenizePosition is called
    pub end_time:         i64,      // Unix timestamp; no new orders after this
    pub resolved:         bool,
    pub winning_outcome:  u8,       // 0=unresolved, 1=YES, 2=NO
    pub admin:            Pubkey,   // only key that can call ResolveMarket
    pub order_count:      u64,
    pub bump:             u8,
}
```

Total size: **212 bytes** (fixed, rent-exempt).

---

## Market Lifecycle

```
CreateMarket (admin)
      │
      ▼
  OPEN — accepts Split, PlaceOrder, CancelOrder, FillOrder
      │
      │  end_time reached
      ▼
  EXPIRED — no new orders; existing orders can still be cancelled
      │
      │  ResolveMarket (admin)
      ▼
  RESOLVED — winning_outcome set; Redeem is now available
```

| State | `resolved` | `winning_outcome` | Allowed instructions |
|-------|-----------|-------------------|----------------------|
| Open | `false` | `0` | Split, Merge, PlaceOrder, CancelOrder, FillOrder |
| Expired | `false` | `0` | CancelOrder, Merge |
| Resolved | `true` | `1` or `2` | Redeem, Merge |

---

## Question Hash

Markets are identified by the SHA-256 hash of the question string. This keeps the Market PDA deterministic — anyone can derive the market pubkey if they know the question text.

Derive a question hash in TypeScript:

```typescript
import { createHash } from "crypto";

function questionHash(question: string): Uint8Array {
  return new Uint8Array(createHash("sha256").update(question).digest());
}

const hash = questionHash("Will BTC be above $100k by end of 2025?");
```

Derive the Market PDA:

```typescript
import { findMarketPda } from "@polymarket-sol/sdk";

const [marketPda] = findMarketPda(hash, PROGRAM_ID);
```

---

## Market PDA Seeds

```
[b"market", question_hash]
```

See [PDA Seeds](../program/pda-seeds.md) for all seed derivations.

---

## Admin Responsibilities

The `admin` field on a market is the only pubkey that can call `ResolveMarket`. There is no other special admin privilege — the admin does not control order matching, and they cannot cancel other users' orders.

The admin is set to the signer of `CreateMarket` and cannot be changed after creation.

| Admin action | Instruction | Constraint |
|-------------|-------------|------------|
| Create market | `CreateMarket` | Admin pays vault ATA rent |
| Resolve market | `ResolveMarket` | `user_ai.key == market.admin` and `!market.resolved` |

---

## Vault

Each market has exactly **one USDC Associated Token Account** (the vault), created at market creation and owned by the `vault_authority` PDA. All collateral from all users in a market flows through this single account.

This design avoids per-user escrow ATAs, keeping ATA rent cost constant at **1 per market** regardless of how many users participate.

See [Collateral](./collateral.md) for the full vault and rent model.

---

## Next Steps

- [Prices & Order Book](./prices-and-orderbook.md)
- [Order Lifecycle](./order-lifecycle.md)
- [Instructions — CreateMarket](../program/instructions.md#createmarket)
