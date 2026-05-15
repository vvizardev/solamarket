# Keeper Economics

> Fill fees, costs, and profitability model for running a keeper bot.

---

## Revenue Sources

A keeper earns from two sources per successful `FillOrder`:

### 1. Fill Fee (USDC-equivalent `no_balance`)

```
fill_fee = fill_size × 5 / 10_000   (5 basis points = 0.05%)
```

The fill fee is credited to `keeper_pos.no_balance` — the keeper's internal collateral balance in the market. To access it as real USDC, the keeper must call `Merge` (which requires also holding `yes_balance`) or use the internal balance to place orders.

> **Current limitation:** The fill fee is credited as `no_balance`, not as direct USDC. A future `WithdrawFee` instruction could allow keepers to withdraw directly to their USDC ATA.

### 2. Order Account Rent (SOL)

When the handler closes a fully-filled `Order` PDA, the rent-exempt lamports (~0.0010 SOL per order) are transferred to the keeper's `UserPosition` account instead of back to the order owner.

Over many fills, this accumulates meaningful SOL revenue.

---

## Costs

| Cost | Unit | Typical amount |
|------|------|----------------|
| Solana base tx fee | SOL | ~0.000005 SOL per signature |
| Priority fee (optional) | SOL | Variable; keeper's choice |
| Failed/raced tx | SOL | Fee burned even if fill already taken |
| RPC provider | USD/month | $0 (free tier) to $50+ (dedicated) |

---

## Profitability Model

```
revenue_per_fill = fill_fee (USDC no_balance) + order_rent × fills_closed (SOL)
cost_per_fill    = tx_base_fee + priority_fee + race_losses
profit_per_fill  = revenue - cost
```

Example at 100 USDC fill, 60 cent market:

```
fill_fee     = 100_000_000 × 5 / 10_000 = 50_000 units = 0.05 USDC
order_rent   = ~0.0010 SOL (if both orders fully filled)
tx_fee       = ~0.000005 SOL
net_SOL      = +0.0010 - 0.000005 ≈ +0.001 SOL  (from rent)
net_USDC     = +0.05 USDC (in no_balance)
```

For small fills (< ~200 USDC), the fill fee in USDC may not justify the opportunity cost of the transaction. The `MIN_FILL_SIZE` setting lets you skip economically unattractive fills.

---

## Competition and Front-Running

Multiple keepers can race on the same crossing. The first transaction to land wins the fill fee; losers pay tx fees for nothing.

**Strategies keepers can use:**
- **Simulation check** — simulate before sending to skip already-filled orders (implemented in `Filler.ts`).
- **Priority fees** — pay higher compute unit prices to get faster inclusion.
- **Dedicated RPC** — lower latency to the validator reduces time to first fill.
- **JIT-style window** — not implemented; future extension modeled after Drift's JIT auction.

The plan acknowledges front-running as an open issue:

> *"Keepers can race on fills. A short JIT window (like Drift's) or commit-reveal scheme could mitigate if needed."*

---

## Comparing to Drift Protocol Keepers

Drift Protocol keepers (fillers) operate under a similar incentive model:

| Dimension | Drift | This project |
|-----------|-------|--------------|
| Fill reward | Dynamic fee + discount token | Flat 5 bps |
| Fee currency | USDC (from taker) | `no_balance` (not direct USDC) |
| Priority fee auction | Yes (revenue share) | Not implemented |
| JIT auction | Yes | Not implemented |
| Cleanup fees | Yes (cancel expired orders) | Not implemented |

---

## Future Improvements

1. **Configurable `fill_fee_bps` per market** — stored as a field on the `Market` account rather than hardcoded to `5`.
2. **Direct USDC withdrawal** — a `WithdrawFee` instruction letting keepers drain `no_balance` to their USDC ATA without needing a matched `yes_balance`.
3. **Priority fee auction** — keepers bid via Solana priority fees; the revenue is shared with the protocol to fund a rebate pool.
4. **GTD cleanup fees** — keepers earn a small fee for cancelling expired GTD orders.

---

## Next Steps

- [Operations](./operations.md)
- [SDK — Fees](../sdk/fees.md)
