# Positions & Tokens

> Internal balance model, YES/NO token semantics, locked amounts, and optional SPL tokenization.

---

## Overview

This project uses an **internal balance model** by default: all YES, NO, and collateral balances are stored as `u64` fields inside a `UserPosition` PDA rather than as separate SPL token accounts. Real SPL tokens are only minted on explicit opt-in via `TokenizePosition`.

This eliminates nearly all per-user ATA rent costs compared to a naïve SPL-first design.

---

## UserPosition Account

One `UserPosition` PDA is created per `(user, market)` pair on the first `Split` call.

```rust
pub struct UserPosition {
    pub discriminant:      u8,
    pub market:            Pubkey,
    pub user:              Pubkey,
    pub yes_balance:       u64,   // free YES shares
    pub no_balance:        u64,   // free NO shares
    pub locked_yes:        u64,   // YES locked in open ask orders
    pub locked_no:         u64,   // (reserved for future NO-side orders)
    pub locked_collateral: u64,   // USDC locked in open bid orders
    pub open_orders:       [Pubkey; 32],
    pub open_order_count:  u8,
    pub bump:              u8,
}
```

Total size: **1131 bytes**.

All balance fields use **6-decimal precision**, matching USDC (1 USDC = 1,000,000 units).

---

## Balance Semantics

### Free balances

`yes_balance` and `no_balance` are unencumbered shares the user can freely sell, merge, or (opt-in) tokenize.

### Locked balances

When an order is placed, the relevant balance is moved from "free" to "locked":

| Order side | What gets locked | Field |
|-----------|-----------------|-------|
| Bid (buy YES) | Collateral (USDC) | `locked_collateral` |
| Ask (sell YES) | YES shares | `locked_yes` |

Locking prevents double-spending — a user cannot place two asks totaling more YES than they own.

When an order is cancelled or fully filled, locked amounts are released or consumed accordingly.

---

## Token Model — Default Path (No SPL Tokens)

```
User deposits 100 USDC
      │
      ▼  Split(100_000_000)
  USDC transferred → market vault ATA
  UserPosition.yes_balance += 100_000_000
  UserPosition.no_balance  += 100_000_000

All trading updates numbers in UserPosition — no ATA needed.

After resolution (YES wins):
  Redeem(100_000_000)
      │
      ▼
  yes_balance -= 100_000_000
  USDC transferred from vault → user's USDC ATA
```

**ATA cost for a regular user: 0 new ATAs** beyond their existing USDC ATA.

---

## Token Model — Optional SPL Tokenization

Users who want to use their outcome shares in external DeFi protocols (LP pools, lending, bridges) can call `TokenizePosition` to convert internal balances into real SPL tokens.

```
TokenizePosition(amount)
  ├─ lazily creates YES/NO mints (if not yet created for this market)
  ├─ creates user YES ATA + user NO ATA  (user pays ~0.004 SOL rent)
  └─ mints `amount` YES tokens and `amount` NO tokens to user's ATAs
  └─ debits yes_balance and no_balance accordingly
```

Once tokenized, those shares become composable SPL tokens and can no longer be used via the internal balance path directly.

---

## ATA Cost Summary

| Account | Count | Payer | When |
|---------|-------|-------|------|
| Market USDC vault ATA | 1 per market | Admin | At `CreateMarket` |
| User USDC ATA | 1 per user (lifetime) | User | Likely already exists |
| YES / NO ATAs | 0 by default; 2 if `TokenizePosition` | User (opt-in) | At `TokenizePosition` |
| Order escrow ATA | 0 — locked amounts are fields | n/a | Never needed |

Compare to a naïve SPL design that would require 2–3 ATAs per user per market (~0.006–0.01 SOL upfront).

---

## Open Orders List

The `UserPosition` account stores a fixed-size array of up to **32 open order pubkeys**. This is used to:

1. Validate that a user doesn't exceed their balance across multiple orders.
2. Allow the `CancelOrder` and `FillOrder` handlers to efficiently remove a closed order from the list.

If a user attempts to place a 33rd simultaneous open order, the transaction fails with `OpenOrdersFull (error 33)`.

---

## Next Steps

- [Collateral (mock USDC)](./collateral.md)
- [Order Lifecycle](./order-lifecycle.md)
- [Instructions — Split / Merge / TokenizePosition](../program/instructions.md#split)
