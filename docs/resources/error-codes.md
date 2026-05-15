# Error Codes

> All `PredictionMarketError` variants with numeric codes and descriptions.

---

## Error Format

On-chain errors are returned as `ProgramError::Custom(code)`. The Solana RPC surfaces them in transaction logs as:

```
Program log: Error: custom program error: 0x14
```

`0x14` = decimal `20` = `InvalidOrderPrice`.

---

## Account Validation Errors (0–9)

| Code | Hex | Name | Description |
|------|-----|------|-------------|
| 0 | 0x0 | `InvalidAccountOwner` | An account's owner is not the expected program or system program. |
| 1 | 0x1 | `InvalidPda` | PDA derivation check failed — the account address does not match the expected seeds. |
| 2 | 0x2 | `MissingRequiredSigner` | A required signer (user, admin, or keeper) did not sign the transaction. |
| 3 | 0x3 | `InvalidDiscriminant` | Account data starts with an unexpected discriminant byte. |

---

## Market Lifecycle Errors (10–19)

| Code | Hex | Name | Description |
|------|-----|------|-------------|
| 10 | 0xA | `MarketAlreadyResolved` | Attempting to resolve a market that is already resolved, or place an order on a resolved market. |
| 11 | 0xB | `MarketExpired` | Attempting to place an order after `end_time` has passed. |
| 12 | 0xC | `MarketNotResolved` | Attempting to redeem before the market has resolved. |
| 13 | 0xD | `InvalidWinningOutcome` | `ResolveMarket` called with an outcome value other than `1` (YES) or `2` (NO). |
| 14 | 0xE | `NotMarketAdmin` | Signer is not `market.admin`; only the admin can resolve. |

---

## Order Errors (20–29)

| Code | Hex | Name | Description |
|------|-----|------|-------------|
| 20 | 0x14 | `InvalidOrderPrice` | Price is outside the valid range (must be 1–9999 basis points). |
| 21 | 0x15 | `InvalidOrderSize` | Order size is zero. |
| 22 | 0x16 | `InvalidOrderSide` | `FillOrder` was called with `bid.side != 0` or `ask.side != 1`. |
| 23 | 0x17 | `MarketMismatch` | The bid and ask orders belong to different markets. |
| 24 | 0x18 | `NoCrossing` | `bid.price < ask.price` — orders do not cross. |
| 25 | 0x19 | `OverFill` | `fill_size` would exceed the remaining size of one of the orders. |
| 26 | 0x1A | `NotOrderOwner` | `CancelOrder` caller is not `order.user`. |

---

## Balance Errors (30–39)

| Code | Hex | Name | Description |
|------|-----|------|-------------|
| 30 | 0x1E | `InsufficientBalance` | User does not have enough free or locked balance for the operation. |
| 31 | 0x1F | `Overflow` | Arithmetic overflow in balance calculation. |
| 32 | 0x20 | `ZeroAmount` | An instruction was called with `amount = 0`. |
| 33 | 0x21 | `OpenOrdersFull` | User already has 32 open orders in this market — the maximum. Cancel an existing order first. |

---

## Event Errors (40–49)

| Code | Hex | Name | Description |
|------|-----|------|-------------|
| 40 | 0x28 | `EventFull` | Event already has 16 markets; no more can be added. |
| 41 | 0x29 | `MarketAlreadyInEvent` | `AddMarketToEvent` called on a market whose `event` field is already set. |
| 42 | 0x2A | `EventAlreadyResolved` | `ResolveEvent` or `AddMarketToEvent` called on an already-resolved event. |
| 43 | 0x2B | `NotEventAdmin` | Signer is not `event.admin`; only the admin can modify or resolve an event. |
| 44 | 0x2C | `EventAdminMismatch` | `market.admin` does not equal `event.admin`; markets and events must share the same admin. |
| 45 | 0x2D | `InvalidMarketIndex` | `winning_index` passed to `ResolveEvent` is ≥ `event.market_count`. |
| 46 | 0x2E | `EventMarketMismatch` | A market account passed to `ResolveEvent` does not match the corresponding `event.markets[i]` pubkey. |
| 47 | 0x2F | `NotExclusiveEvent` | `ResolveEvent` called on an event where `is_exclusive = false`; use per-market `ResolveMarket` instead. |

---

## Handling Errors in TypeScript

```typescript
import { SendTransactionError } from "@solana/web3.js";

try {
  await sendAndConfirmTransaction(connection, tx, [signer]);
} catch (err) {
  if (err instanceof SendTransactionError) {
    const logs = err.logs ?? [];
    const match = logs.join("\n").match(/custom program error: 0x([0-9a-fA-F]+)/);
    if (match) {
      const code = parseInt(match[1], 16);
      console.error("Program error code:", code);
      // Map to PredictionMarketError name using the table above
    }
  }
}
```

---

## Rust Definition

```rust
// program/src/error.rs
pub enum PredictionMarketError {
    InvalidAccountOwner    = 0,
    InvalidPda             = 1,
    MissingRequiredSigner  = 2,
    InvalidDiscriminant    = 3,

    MarketAlreadyResolved  = 10,
    MarketExpired          = 11,
    MarketNotResolved      = 12,
    InvalidWinningOutcome  = 13,
    NotMarketAdmin         = 14,

    InvalidOrderPrice      = 20,
    InvalidOrderSize       = 21,
    InvalidOrderSide       = 22,
    MarketMismatch         = 23,
    NoCrossing             = 24,
    OverFill               = 25,
    NotOrderOwner          = 26,

    InsufficientBalance    = 30,
    Overflow               = 31,
    ZeroAmount             = 32,
    OpenOrdersFull         = 33,

    EventFull              = 40,
    MarketAlreadyInEvent   = 41,
    EventAlreadyResolved   = 42,
    NotEventAdmin          = 43,
    EventAdminMismatch     = 44,
    InvalidMarketIndex     = 45,
    EventMarketMismatch    = 46,
    NotExclusiveEvent      = 47,
}
```
