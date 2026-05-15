# Keeper Economics

> Taker-fee splits, keeper rewards, costs, and profitability for running a keeper bot.

---

## Revenue Sources

On each successful `FillOrder`, the keeper earns **`keeper_reward`**: a configured fraction of the **taker fee** (Polymarket-style probability curve) in USDC-equivalent `no_balance`.

```
taker_fee     = fill_cost × bid.price × (10_000 − bid.price) × taker_curve_numer
                / (taker_curve_denom × 10_000 × 10_000)

keeper_reward = taker_fee × keeper_reward_of_taker_bps / 10_000
```

`keeper_reward` is credited to **`keeper_pos.no_balance`**.

If the market routes **`treasury_share`** to the same `UserPosition` as the keeper (because `fee_recipient_user` is the keeper’s wallet), accounts **#6** and **#7** in `FillOrder` may be the **same** PDA — credits for keeper reward, treasury, and **maker_fee** still add correctly on one balance.

The **maker rebate** accrues to the **maker**, not the keeper.

> **Current limitation:** Rewards are internal `no_balance`, not SPL USDC, until a future **`WithdrawFee`** instruction lands.

### Order Account Rent (SOL) — Order Creator

When a fully-filled `Order` PDA closes, rent lamports (~0.0010 SOL) return to the **order creator**. That is **not** keeper revenue.

---

## Costs

| Cost | Unit | Typical amount |
|------|------|----------------|
| Solana base tx fee | SOL | ~0.000005 SOL per signature |
| Priority fee (optional) | SOL | Variable |
| Failed/raced tx | SOL | Paid even if another keeper won the fill |
| RPC provider | USD/month | Free tier to dedicated |

---

## Profitability Model

```
revenue_per_fill ≈ keeper_reward
                   (+ treasury_share + maker_fee when fee recipient is the keeper wallet)
cost_per_fill    = tx_base_fee + priority_fee + race_losses
profit_per_fill  = revenue − cost
```

**Worked example:** `fill_size = 100_000_000` (100 YES shares), `bid.price = 6000`, `taker_curve_numer/denom = 1/100`, `keeper_reward_of_taker_bps = 500`:

```
fill_cost = 100_000_000 × 6000 / 10_000 = 60_000_000    (60 USDC notional)
curve     = 6000 × 4000 = 24_000_000
taker_fee = 60_000_000 × 24_000_000 × 1 / (100 × 10_000 × 10_000)
          = 144_000                                     (0.144 USDC)

keeper_reward = 144_000 × 500 / 10_000 = 7_200          (0.0072 USDC)
```

Most of **`taker_fee`** can go to **maker rebate** and **treasury** depending on `Market` params; the keeper’s line item may be modest unless `keeper_reward_of_taker_bps` is set high or the keeper wallet is also the fee recipient.

---

## Competition and Front-Running

Multiple keepers can race the same cross. First tx to land gets the work; others pay fees for nothing.

**Mitigations:** simulation before send, priority fees, low-latency RPC. A future JIT-style window could reduce races (see [Program overview](../program/overview.md)).

---

## Comparing to Drift Protocol Keepers

| Dimension | Drift | This project (spec) |
|-----------|-------|---------------------|
| Fill reward | Dynamic taker fee / incentives | Shares of **taker_fee** + optional treasury routing |
| Fee currency | Often direct USDC | Internal `no_balance` until `WithdrawFee` |
| Priority fee auction | Yes | Not specified |
| Cleanup fees | Expired orders | Not implemented |

---

## Future Improvements

1. **`WithdrawFee`** — move fee balances to a USDC ATA without `Merge`.
2. **Per-market tuning** — `taker_curve_*`, rebate, and keeper bps chosen at `CreateMarket`.
3. **GTD cleanup fees** — reward keepers for cancelling expired orders.

---

## Next Steps

- [Operations](./operations.md)
- [SDK — Fees](../sdk/fees.md)
