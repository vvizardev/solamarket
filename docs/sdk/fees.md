# Fees

> Fill fee mechanics, keeper incentives, and cost breakdown.

---

## Fee Structure

This project charges a single protocol fee: a **fill fee paid to the keeper** at the time of order matching.

| Fee type | Rate | Payer | Recipient |
|----------|------|-------|-----------|
| Fill fee | 5 bps (0.05%) | Ask side (seller) | Keeper's `UserPosition.no_balance` |
| Solana tx fee | ~5,000 lamports/signature | Keeper wallet (SOL) | Solana validators |
| Order rent | ~0.0010 SOL | Order placer | Returned on close; excess to keeper |

There are **no maker fees**, **no taker fees**, and **no platform fees** — only the 5 bps fill fee.

---

## Fill Fee Calculation

```
fill_cost     = fill_size × bid.price / 10_000
fill_fee      = fill_size × 5 / 10_000
ask_proceeds  = fill_cost - fill_fee
```

Example — fill 100 YES shares at bid price 6000 (0.60):

```
fill_cost    = 100,000,000 × 6000 / 10,000 = 60,000,000 (60 USDC)
fill_fee     = 100,000,000 × 5    / 10,000 =     50,000  (0.05 USDC)
ask_proceeds = 60,000,000 - 50,000         = 59,950,000  (59.95 USDC)
```

| Party | Balance change |
|-------|----------------|
| Bid (buyer) | `locked_collateral −60 USDC`, `yes_balance +100 shares` |
| Ask (seller) | `locked_yes −100 shares`, `no_balance +59.95 USDC` |
| Keeper | `no_balance +0.05 USDC` |

The SDK constant:

```typescript
export const FILL_FEE_BPS = 5n; // 5 basis points
```

---

## Fee Precision

Fees are computed with integer arithmetic (no floating point). For very small fills, the result of `fill_size × 5 / 10_000` may truncate to **zero** — in that case no fee is charged. This is intentional and consistent with `saturating_sub` / `unwrap_or(0)` in the program.

The minimum fill size for a non-zero fee is:

```
min_fill_size_for_fee = 10_000 / 5 = 2_000 units (0.002 USDC)
```

The keeper's `minFillSize` config (default: `1000` units) may be set below this threshold. Fills below the fee minimum incur no keeper reward but still execute correctly.

---

## Order Rent as Additional Keeper Revenue

When the `FillOrder` handler closes a **fully-filled** Order PDA, the account's rent-exempt lamports are transferred to the **keeper's UserPosition account** (not back to the original order placer):

```rust
// From fill_order.rs
let lamps = bid_order_ai.lamports();
**bid_order_ai.lamports.borrow_mut()   = 0;
**keeper_pos_ai.lamports.borrow_mut() += lamps;
```

An Order account holds ~0.0010 SOL in rent. A keeper who fills many orders accumulates this SOL in their UserPosition, which they can recover via a future withdrawal instruction.

---

## Keeper Cost Model

A keeper pays SOL tx fees for every `FillOrder` transaction it submits. Revenue comes from:

1. **Fill fee** (0.05% of fill size in USDC-equivalent `no_balance`)
2. **Order account rent** (SOL, ~0.001 per fully-filled order)

At current devnet SOL prices, this model is illustrative. On mainnet, profitability depends on:

- Average fill size
- Number of competing keepers (race cost)
- Solana tx fee / priority fee levels

See [Keeper — Economics](../keeper/economics.md) for a detailed breakdown.

---

## Polymarket Fee Comparison

Polymarket's fee structure is significantly more complex:

| Dimension | Polymarket | This project |
|-----------|-----------|--------------|
| Taker fee | 0.03–0.07 × `feeRate × p × (1−p)` per market category | 0 |
| Maker fee | 0 (makers earn rebates) | 0 |
| Fill/matching fee | None | 5 bps to keeper |
| Fee recipient | Protocol treasury → maker rebate pool | Keeper `no_balance` |

---

## Next Steps

- [Keeper — Economics](../keeper/economics.md)
- [Orders](./orders.md)
