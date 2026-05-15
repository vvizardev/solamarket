# Resolution

> Admin resolution, winning outcome, payout mechanics, and future oracle integration.

---

## How Markets Resolve

Market resolution is a two-step process:

1. **Admin calls `ResolveMarket`** — sets `market.resolved = true` and `market.winning_outcome`.
2. **Winners call `Redeem`** — exchange their winning outcome balance for USDC at 1:1.

---

## ResolveMarket Instruction

The admin provides a 1-byte outcome value:

| Value | Meaning |
|-------|---------|
| `1` | YES wins — YES holders can redeem |
| `2` | NO wins — NO holders can redeem |

Constraints enforced on-chain:

- Signer must equal `market.admin` (returns `NotMarketAdmin` error otherwise).
- Market must not already be resolved (returns `MarketAlreadyResolved` error otherwise).
- Outcome must be `1` or `2` (returns `InvalidWinningOutcome` for any other value).

There is no cooldown, no dispute window, and no oracle check in the current implementation. Admin key security is the sole guarantee of correct resolution. See [Future: Oracle Integration](#future-oracle-integration) below.

---

## Redeem Instruction

After resolution, winners exchange their outcome shares for USDC:

```
If winning_outcome == 1 (YES):
  yes_balance -= amount
  vault → amount USDC → user's USDC ATA

If winning_outcome == 2 (NO):
  no_balance -= amount
  vault → amount USDC → user's USDC ATA
```

Redemption rate is always **1:1** (1 winning share = 1 USDC unit = 0.000001 USDC at 6 decimals, or 1 USDC for 1,000,000 units).

### Loser payout

Holders of the **losing side** receive nothing via `Redeem`. However, they can still call `Merge` before or after resolution to recover collateral for any paired YES+NO balance they hold:

```
Merge(amount)  — available any time
  yes_balance -= amount
  no_balance  -= amount
  vault → amount USDC → user's USDC ATA
```

---

## What Happens to Unfilled Orders at Resolution?

The program does not automatically cancel open orders at resolution. Users must cancel their own orders manually after a market resolves to:

1. Recover locked collateral (bid orders) or locked YES (ask orders).
2. Reclaim Order PDA rent.

A future extension could allow keepers to submit `CancelOrder` for expired or post-resolution orders and earn a cleanup fee.

---

## Scripts

```bash
# Resolve a market from the CLI
pnpm ts-node scripts/resolve-market.ts
```

The `resolve-market.ts` script reads the market pubkey and outcome from environment variables (or prompts interactively) and submits a `ResolveMarket` transaction signed by `wallet/admin.json`.

---

## Future: Oracle Integration

The current design uses a trusted admin keypair. Two upgrade paths are planned:

### 1. Switchboard VRF / Pyth Price Feeds

For price-based markets (e.g., "Will BTC > $100k?"), a Pyth price account can be passed as a read-only account to `ResolveMarket`. The on-chain handler would read the price from the oracle account and determine the outcome automatically, removing human admin discretion.

### 2. UMA Optimistic Oracle

A dispute window and bond mechanism (similar to UMA's DVM) can be added where:
- Anyone can propose a resolution with a USDC bond.
- A dispute period allows challengers to post a counter-bond.
- After the window, the undisputed proposal finalizes.

---

## Next Steps

- [Redeem instruction details](../program/instructions.md#redeem)
- [SDK — Outcome Tokens](../sdk/outcome-tokens.md)
