# Negative Risk Markets

> Complete sets, negative-risk positions, and atomic cross-market collateral conversion for exclusive multi-outcome events.

---

## Overview

A **Negative Risk Market** is a pricing and collateral pattern that emerges from an **exclusive multi-outcome event** — one where exactly one market in the group resolves YES and all others resolve NO.

Because exactly one outcome MUST win, holding one YES share from every market in the event (a **complete set**) is risk-free: it is always redeemable for exactly 1 USDC. This guaranteed redemption value is the foundation of the negative-risk model.

The core insight is:

```
Complete Set  =  YES(market 1) + YES(market 2) + … + YES(market N)
             ≡  1 USDC  (risk-free, since exactly one must resolve YES)

Negative-Risk Position in market X
             =  Complete Set  −  YES(market X)
             ≡  NO position in market X
             (pays 1 USDC if any outcome other than X wins)
```

This is not a new account type — it is a **collateral accounting technique** built on top of exclusive `Event` accounts and the existing `Split` / `Merge` mechanics.

---

## Why It Matters

In a standard binary market, selling a NO share requires a counterparty to buy it. In an exclusive event, a trader who wants exposure to "NOT outcome X" can instead:

1. Split 1 USDC into a complete set (1 YES share per market).
2. Sell or keep the YES share for market X.
3. Hold the remaining N−1 YES shares as a synthetic NO position.

This has two advantages over a traditional NO bid:

| Dimension | Traditional NO bid | Negative-Risk Position |
|-----------|-------------------|----------------------|
| Counterparty required | Yes | No (split is instant) |
| Capital efficiency | Locks full collateral | Unlocks partial collateral by selling the unwanted YES |
| Composability | Single-market | Spans the entire event |

---

## Complete Set

A **complete set** is a set of exactly one YES share from every market in an exclusive event. Because exactly one market resolves YES, a complete set always pays out 1 USDC.

### Split into a Complete Set

Deposit `N × amount` USDC (where N = number of markets in the event) and receive `amount` YES shares in each market:

```typescript
import { splitCompleteSet } from "@polymarket-sol/sdk";

// Exclusive event with 3 markets (e.g. Trump / Biden / Other)
const ix = splitCompleteSet({
  eventPda,
  marketPdas,          // [trumpMarket, bidenMarket, otherMarket]
  userPositionPdas,    // one UserPosition PDA per market
  amount: 1_000_000n,  // 1 USDC = 1_000_000 units
  user:   userKeypair.publicKey,
  programId: PROGRAM_ID,
});
```

After this call, the user's `UserPosition` in each market has `yes_balance` increased by `1_000_000` and 1 USDC has been deducted from their wallet per market (3 USDC total for a 3-market event).

### Merge a Complete Set

The inverse: return `amount` YES shares from every market in the event and receive `N × amount` USDC back:

```typescript
import { mergeCompleteSet } from "@polymarket-sol/sdk";

const ix = mergeCompleteSet({
  eventPda,
  marketPdas,
  userPositionPdas,
  amount: 1_000_000n,
  user:   userKeypair.publicKey,
  programId: PROGRAM_ID,
});
```

On-chain constraints:
- The event must be `is_exclusive = true`.
- The user must have ≥ `amount` free `yes_balance` in every market in the event.
- All N markets must be unresolved.

---

## Negative-Risk Position

A negative-risk position is a complete set from which the YES share of one specific market has been removed (sold or transferred). What remains pays 1 USDC if any outcome other than the removed one wins.

### Building a Negative-Risk Position

```typescript
// Step 1 — split 1 USDC into a complete set across all 3 markets
await splitCompleteSet({ eventPda, marketPdas, amount: 1_000_000n, ... });

// Step 2 — sell the YES share for the outcome you do NOT want exposure to
//           (e.g. sell Trump YES at current market price)
await placeOrder({
  marketPda: trumpMarketPda,
  side:  "ask",          // sell YES
  price: 5500n,          // 55 cents (55% probability)
  size:  1_000_000n,
  ...
});

// You now hold: Biden YES + Other YES
// This position pays 1 USDC if anyone other than Trump wins.
```

### Economic Equivalence

| Position | Payout if Trump wins | Payout if other wins |
|----------|---------------------|---------------------|
| NO share in Trump market (traditional) | 0 | 1 USDC |
| Negative-Risk (Biden YES + Other YES) | 0 | 1 USDC |

The two are economically identical in an exclusive event. The negative-risk version is constructed without ever placing a NO bid — it uses `SplitCompleteSet` instead.

---

## Cross-Market Conversion

Once a user holds YES shares across multiple markets of an exclusive event, they can rebalance without touching the order book by merging and re-splitting:

```
User holds:  2 YES(Trump) + 1 YES(Biden)
Goal:        1 YES(Trump) + 1 YES(Biden) + 1 YES(Other)

Step 1 — merge 1 complete set using 1 YES from each market they already have:
  MergeCompleteSet(amount=1) → +1 USDC (but user only has 1 Biden YES, not 1 Other YES)
  → This path requires a full complete set; partial sets must go through the order book.

Alternative (direct rebalance via order book):
  Sell 1 YES(Trump), buy 1 YES(Other) — two orders, one YES left each.
```

Tip: `SplitCompleteSet` and `MergeCompleteSet` require balanced quantities. For uneven rebalancing, standard `PlaceOrder` is still needed.

---

## On-Chain Implementation

Negative-risk mechanics do not require new account types. They are built on top of the existing `Event`, `Market`, and `UserPosition` accounts using two new composite instructions:

### `SplitCompleteSet` (instruction 12)

Atomically calls the equivalent of `Split(amount)` on every market in the event in a single transaction.

| Account | Role |
|---------|------|
| `event` | Verifies `is_exclusive = true` and reads `markets[]` |
| `market[0..N]` | Each market PDA (must match `event.markets[]`) |
| `user_position[0..N]` | One `UserPosition` PDA per market |
| `vault[0..N]` | Each market's USDC vault ATA |
| `user_usdc_ata` | Source of collateral |
| `user` | Signer |

On-chain logic (per market `i`):

```
vault[i] ← amount USDC  (from user_usdc_ata)
user_position[i].yes_balance += amount
user_position[i].no_balance  += amount
```

Total USDC deducted: `amount × N`.

### `MergeCompleteSet` (instruction 13)

Atomically calls the equivalent of `Merge(amount)` on every market in the event.

On-chain constraints:

```
for each market i:
  require user_position[i].yes_balance >= amount
  require user_position[i].no_balance  >= amount (or allow asymmetric merge — TBD)
  user_position[i].yes_balance -= amount
  user_position[i].no_balance  -= amount
  vault[i] → amount USDC → user_usdc_ata
```

Total USDC returned: `amount × N`.

---

## Price Relationships in Exclusive Events

In a well-functioning exclusive event, the sum of all YES prices equals 1 (the price of a complete set is exactly 1 USDC):

```
P(Trump YES) + P(Biden YES) + P(Other YES)  ≈  1.00
```

Any deviation from this sum creates a risk-free arbitrage opportunity:

| Condition | Arbitrage action |
|-----------|-----------------|
| `ΣP > 1` (overpriced) | Sell all YES shares; buy a complete set back for < ΣP |
| `ΣP < 1` (underpriced) | Buy a complete set via `SplitCompleteSet`; sell individual YES shares above cost |

Keepers or market makers can automate this arb using `SplitCompleteSet` / `MergeCompleteSet` + order book fills, keeping prices in equilibrium without manual intervention.

---

## Interaction with Resolution

When an exclusive event resolves via `ResolveEvent`:

- The winning market resolves YES → YES holders redeem 1 USDC per share via `Redeem`.
- All other markets resolve NO → losing YES holders receive 0 via `Redeem`.
- A complete-set holder nets exactly 1 USDC regardless of which outcome wins (from the one winning YES share).
- A negative-risk holder nets 1 USDC from any of the N−1 winning YES shares they held, 0 if the excluded outcome wins.

### Complete-set after resolution

After `ResolveEvent`, `MergeCompleteSet` is no longer valid (markets are resolved). Users holding a complete set should:

1. Call `Redeem` on the one winning market (the single YES share that resolves 1:1 to USDC).
2. The N−1 losing YES shares are worthless via `Redeem`; they may call `Merge` on each individual market to recover collateral for any paired YES+NO they hold.

---

## Relationship to Standard Split / Merge

| Operation | Scope | Instruction |
|-----------|-------|-------------|
| `Split` | Single market | `Split(amount)` — deposit USDC, receive YES + NO |
| `Merge` | Single market | `Merge(amount)` — return YES + NO, receive USDC |
| `SplitCompleteSet` | Entire exclusive event | Deposit N × USDC, receive YES in every market |
| `MergeCompleteSet` | Entire exclusive event | Return YES from every market, receive N × USDC |

Standard `Split` / `Merge` still work on individual markets within an event — `SplitCompleteSet` is a convenience wrapper that batches all N splits atomically.

---

## SDK Reference

```typescript
import {
  splitCompleteSet,
  mergeCompleteSet,
  findEventPda,
  findMarketPda,
  findUserPositionPda,
  fetchEvent,
} from "@polymarket-sol/sdk";

// Fetch event to get all market PDAs
const event = await fetchEvent(connection, eventPda);
const marketPdas = event.markets.slice(0, event.marketCount);

// Derive per-market user position PDAs
const userPositionPdas = marketPdas.map(
  (mPda) => findUserPositionPda(mPda, user, PROGRAM_ID)[0]
);

// Split 5 USDC into a complete set (5 YES per market, 3-market event → 15 USDC total)
const splitIx = splitCompleteSet({
  eventPda,
  marketPdas,
  userPositionPdas,
  amount: 5_000_000n,
  user,
  programId: PROGRAM_ID,
});

await sendAndConfirmTransaction(connection, new Transaction().add(splitIx), [userKeypair]);

// Later: merge back
const mergeIx = mergeCompleteSet({
  eventPda,
  marketPdas,
  userPositionPdas,
  amount: 5_000_000n,
  user,
  programId: PROGRAM_ID,
});
```

---

## Constraints Summary

| Rule | Enforced by |
|------|------------|
| Event must be `is_exclusive = true` | On-chain → `EventNotExclusive` |
| All markets must be unresolved at split/merge time | On-chain → `MarketAlreadyResolved` |
| `market_count` PDAs must be passed (no partial sets) | On-chain → `InvalidMarketCount` |
| Market PDAs must match `event.markets[]` order | On-chain → `MarketNotInEvent` |
| User must have ≥ `amount` `yes_balance` in each market for `MergeCompleteSet` | On-chain → `InsufficientBalance` |

---

## Next Steps

- [Events — Multi-Market Grouping](./events.md)
- [Resolution — ResolveEvent](./resolution.md)
- [Instructions — SplitCompleteSet / MergeCompleteSet](../program/instructions.md#splitcompleteset)
- [Positions & Tokens](./positions-and-tokens.md)
