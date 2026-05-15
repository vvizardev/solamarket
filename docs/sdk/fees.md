# Fees

> Taker fee (Polymarket-style curve), optional maker fee, maker rebate, keeper reward, and cost breakdown.

---

## Fee Structure

Fees are charged on **`FillOrder`**. The design mirrors **Polymarket’s economics** in three layers:

| Component | Typical role | Payer | Recipient |
|-----------|----------------|-------|-----------|
| **Taker fee** | Probability-weighted curve \( \propto p(1-p) \) | **Taker** | Split: **maker rebate**, **keeper reward**, **treasury** |
| **Maker fee** | Flat bps on notional (usually **0**) | **Maker** | Treasury / protocol (same `fee_recipient` position) |
| **Maker rebate** | Share of **taker fee** | — | **Maker** (`no_balance`) |
| **Keeper reward** | Configurable share of **taker fee** | — | Keeper `UserPosition.no_balance` |
| Solana tx fee | Per signature | Keeper wallet (SOL) | Validators |
| Order rent | Per `Order` PDA | Order placer | Returned to placer on full fill or cancel |

**Legacy note:** Markets MAY set `taker_curve_numer = 0` and use a separate **minimum keeper reward** path (see [Keeper — Economics](../keeper/economics.md)) so behavior stays close to the old flat **5 bps** fill fee during migration.

---

## Maker vs Taker (resting = maker)

On each fill, exactly one resting order is the **maker** and the other side is the **taker**:

1. Compare `bid_order.created_at` and `ask_order.created_at` (both `i64` on-chain).
2. **Older timestamp = maker**, **newer = taker**.
3. **Tie-break (required determinism):** if timestamps are equal, compare `bid_order.key` and `ask_order.key` as `Pubkey` byte arrays lexicographically — **smaller pubkey is maker**.

The keeper MUST pass `(bid_order, ask_order)` account ordering as today; the program applies the maker/taker rule from timestamps + tie-break only (never from account order).

---

## Notional & Execution Price

All fee math uses the **same notional** as the current fill path:

```
fill_cost          = fill_size × bid.price / 10_000
execution_price    = bid.price                    // bps 1–9999 (“p” in bps form)
curve              = execution_price × (10_000 − execution_price)
```

`fill_cost` is in **collateral units** (USDC 6 decimals) for the YES leg of the trade. Integer division floors unless stated otherwise.

---

## Taker Fee (Polymarket-style curve)

The taker pays a fee that **scales with** `p(1-p)` via integer-friendly bps:

```
taker_fee = fill_cost × curve × taker_curve_numer
            / (taker_curve_denom × 10_000 × 10_000)
```

| Field | Location | Meaning |
|-------|-----------|---------|
| `taker_curve_numer` | `Market` | Scales max fee at 50¢ |
| `taker_curve_denom` | `Market` | Must be > 0 when curve fees are enabled |

At **p = 0.50** (`execution_price = 5000`), `curve = 25_000_000`. The ratio `curve / (10_000 × 10_000) = 0.25`, so

```
taker_fee_max_at_mid ≈ fill_cost × (taker_curve_numer / taker_curve_denom) × 1/4
```

**Calibration example:** `numer/denom = 1/100` → **~25 bps of notional** at a 50/50 market:

```
taker_fee ≈ fill_cost × 25_000_000 × 1 / (100 × 10_000 × 10_000)
          = fill_cost × 25 / 10_000
```

Use `u128` intermediates in the program to avoid overflow (`fill_cost × curve` can exceed `u64`).

---

## Maker Fee (optional flat bps)

If `maker_fee_bps > 0`, the **maker** pays a **flat** fee proportional to `fill_cost`:

```
maker_fee = fill_cost × maker_fee_bps / 10_000
```

By default `maker_fee_bps = 0`, matching Polymarket (makers earn rebates, not pay an extra fee).

---

## Splitting the Taker Fee: Rebate, Keeper, Treasury

After `taker_fee` is computed, it is split **in collateral units**:

```
maker_rebate    = taker_fee × maker_rebate_of_taker_bps / 10_000
keeper_reward   = taker_fee × keeper_reward_of_taker_bps / 10_000
treasury_share  = taker_fee − maker_rebate − keeper_reward
```

**Constraints (program invariants):**

- `maker_rebate_of_taker_bps + keeper_reward_of_taker_bps <= 10_000`.
- **Production:** `fee_recipient_user` MUST be non-default so **treasury** and **maker_fee** have an explicit destination (`FillOrder` account #7). If you intentionally colocate treasury with the keeper wallet, set `fee_recipient_user` to that wallet and pass the same `UserPosition` for indices #6 and #7.
- Any integer **dust** stays on **treasury** (or is defined by the program’s rounding policy).

---

## Balance updates on FillOrder

Let:

- `fill_cost`, `taker_fee`, `maker_fee`, `maker_rebate`, `keeper_reward`, `treasury_share` be as above.
- **Maker** and **taker** are identified from the maker/taker rule.

### Case A — Taker is **bid** (buyer), maker is **ask** (seller)

| Party | Change |
|-------|--------|
| **Taker bid** | `locked_collateral -= fill_cost + taker_fee`; `yes_balance += fill_size` |
| **Maker ask** | `locked_yes -= fill_size`; `no_balance += fill_cost − maker_fee + maker_rebate` |
| **Keeper** | `no_balance += keeper_reward` |
| **Treasury position** | `no_balance += treasury_share + maker_fee` |

### Case B — Taker is **ask** (seller), maker is **bid** (buyer)

| Party | Change |
|-------|--------|
| **Maker bid** | `locked_collateral -= fill_cost − maker_rebate`; `yes_balance += fill_size` *(rebate reduces effective price)* |
| **Taker ask** | `locked_yes -= fill_size`; `no_balance += fill_cost − taker_fee − maker_fee` |
| **Keeper** | `no_balance += keeper_reward` |
| **Treasury position** | `no_balance += treasury_share + maker_fee` |

**YES conservation / parity:** The YES leg still moves `fill_size` from ask locked/free to bid; fee accounting only adjusts **collateral / NO-equivalent** legs so total internal USDC semantics stay consistent with vault-backed `Split` / `Merge`.

---

## Fee Flow Diagram

```
Taker pays taker_fee (extra debit or reduced proceeds)
        │
        ├─► MakerRebate  ──► maker UserPosition.no_balance
        ├─► KeeperReward ──► keeper UserPosition.no_balance
        └─► Treasury     ──► fee_recipient UserPosition.no_balance

Maker fee (if any) ────────────────────► fee_recipient UserPosition.no_balance
```

Order rent on closed `Order` PDAs still returns to **each order’s placer** (unchanged).

---

## SDK Constants & Types

```typescript
/** BPS denominator for all bps fields */
export const BPS = 10_000n;

/** Example: ~25 bps of notional at p=0.5 when curve uses numer/denom = 1/100 */
export const DEFAULT_TAKER_CURVE_NUMER = 1n;
export const DEFAULT_TAKER_CURVE_DENOM = 100n;

export function executionCurveBps(priceBps: bigint): bigint {
  return priceBps * (BPS - priceBps);
}

export function takerFee(
  fillCost: bigint,
  priceBps: bigint,
  numer: bigint,
  denom: bigint,
): bigint {
  if (numer === 0n || denom === 0n) return 0n;
  const curve = executionCurveBps(priceBps);
  return (fillCost * curve * numer) / (denom * BPS * BPS);
}
```

---

## Fee Precision

- All amounts use **integer** collateral units; fractional USDC is truncated toward zero.
- **Empty fees:** Any `*_fee` may round to **0** on tiny fills; fills remain valid.
- **Overflow:** Implementations MUST widen to `u128` before dividing when computing `taker_fee`.

---

## Polymarket Comparison

| Dimension | Polymarket | This project (spec) |
|-----------|-----------|---------------------|
| Taker curve | \(\approx k \cdot p(1-p)\) on notional | Same shape via `curve = p(1-p)` in bps |
| Maker fee | 0 in public markets | `maker_fee_bps` default 0 |
| Maker rebate | From taker fee / treasury | `maker_rebate_of_taker_bps` of `taker_fee` |
| Treasury | Central | `fee_recipient` `UserPosition` per market |
| Keeper | Off-chain infra | `keeper_reward_of_taker_bps` (can be 0; rely on priority markets later) |

---

## Fee Roadmap (implementation)

1. **`WithdrawFee` instruction** — move accrued `no_balance` from treasury / keeper to USDC ATA without a matched `Merge`.
2. **Per-category fee caps** — mirror Polymarket’s min/max fee rate by clamping `taker_curve_*` when creating markets.
3. **Affiliate / referral** — optional extra account in `FillOrder` (out of scope here).

---

## Next Steps

- [Keeper — Economics](../keeper/economics.md)
- [Program — Accounts](../program/accounts.md#market-account) (fee fields)
- [Orders](./orders.md)
