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

## Fee Flow

The following shows exactly where each USDC unit goes when `FillOrder` executes.

### Step 1 — Bid side (buyer pays `fill_cost`)

```
Bid User
  locked_collateral -= fill_cost
                           │
                           ▼
                      FillOrder (on-chain)
                           │
                           ▼
Bid User
  yes_balance += fill_size
```

### Step 2 — Ask side (seller receives `fill_cost − fill_fee`)

```
Ask User
  locked_yes -= fill_size
                           │
                           ▼
                      FillOrder (on-chain)
                           │  deducts fill_fee
                           ▼
Ask User
  no_balance += fill_cost - fill_fee
```

### Step 3 — Keeper earns fill fee + order rent

```
FillOrder (on-chain)
  ├─ keeper.no_balance  += fill_fee           (USDC-equiv)
  └─ keeper.UserPosition += order_rent lamports  (SOL, if fully filled)

Keeper (later)
  ├─ Merge(yes_balance + no_balance) → collateral_balance   [current path]
  └─ WithdrawFee → keeper USDC ATA                          [future instruction]
```

### Balance summary per fill

| Party | Balance change |
|-------|----------------|
| Bid (buyer) | `locked_collateral −fill_cost`, `yes_balance +fill_size` |
| Ask (seller) | `locked_yes −fill_size`, `no_balance +(fill_cost − fill_fee)` |
| Keeper | `no_balance +fill_fee`, `+~0.001 SOL` (rent, fully-filled orders only) |
| Solana validators | `+~0.000005 SOL` tx fee (paid by keeper wallet) |

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
| Fill / matching fee | None | 5 bps to keeper |
| Fee recipient | Protocol treasury → maker rebate pool | Keeper `UserPosition.no_balance` |
| Fee currency | USDC (direct) | `no_balance` (internal; requires `Merge` to access as USDC) |
| Order rent | N/A (EVM gas model) | ~0.001 SOL recycled to keeper per fully-filled PDA |
| Affiliate / referral fee | Yes (via API key) | Not implemented |

**Key architectural difference:** Polymarket's taker fee is probability-weighted — it is highest at 50/50 markets and falls toward zero at the extremes (`p × (1−p)` peaks at `p = 0.5`). Revenue flows to a treasury that funds maker rebates. This project's flat 5 bps goes directly to the keeper doing the matching work; there is no treasury or rebate pool.

---

## Comparison with Drift Protocol Keepers

Drift Protocol uses a similar keeper incentive model for its DLOB. Key differences:

| Dimension | Drift | This project |
|-----------|-------|--------------|
| Fill reward | Dynamic fee + discount token | Flat 5 bps |
| Fee currency | USDC (direct from taker fee) | `no_balance` (not direct USDC) |
| Priority fee auction | Yes — revenue shared with protocol | Not implemented |
| JIT auction window | Yes | Not implemented |
| Cleanup fees (expired orders) | Yes | Not implemented |

---

## Fee Roadmap

These improvements are planned but not yet implemented:

1. **Configurable `fill_fee_bps` per market** — stored as a field on the `Market` account rather than hardcoded to `5`. Allows markets to compete on fee rates.
2. **`WithdrawFee` instruction** — lets keepers drain `no_balance` directly to their USDC ATA without needing a matched `yes_balance` for `Merge`.
3. **Priority fee auction** — keepers bid via Solana compute unit price; revenue share with the protocol funds a maker rebate pool (converging toward the Polymarket model).
4. **GTD cleanup fees** — keepers earn a small fee for cancelling expired good-till-date orders, mirroring Drift's cleanup incentive.

See [Keeper — Economics](../keeper/economics.md) for profitability modelling and the front-running discussion.

---

## Next Steps

- [Keeper — Economics](../keeper/economics.md)
- [Orders](./orders.md)
