# Prices & Order Book

> Price representation, DLOB structure, spread, and crossing logic.

---

## Price Representation

Prices are expressed in **basis points** — integers from **1 to 9999**, representing probabilities from 0.01% to 99.99%.

| Basis points | Implied probability | USDC value per 1 YES share |
|-------------|--------------------|-----------------------------|
| 1000 | 10% | $0.10 |
| 5000 | 50% | $0.50 |
| 7500 | 75% | $0.75 |
| 9999 | 99.99% | $0.9999 |

A price of **6000** means "I think YES has a 60% chance of happening" — the buyer expects to pay 0.60 USDC for each YES share that is worth 1.00 USDC if correct.

This is equivalent to Polymarket's price representation (0–1 decimal), but stored as integers to avoid floating-point issues on-chain.

### Converting prices

```typescript
// Basis points → decimal
const decimal = Number(price) / 10_000;   // 6000 → 0.60

// Decimal → basis points
const bps = BigInt(Math.round(decimal * 10_000));  // 0.60 → 6000n
```

---

## Bids and Asks

| Side | `side` byte | Meaning | Price semantics |
|------|-------------|---------|-----------------|
| Bid | `0` | Buy YES (pay collateral, receive YES tokens) | Higher = more aggressive |
| Ask | `1` | Sell YES (give YES tokens, receive collateral) | Lower = more aggressive |

Selling YES is equivalent to buying NO:

- If you own 100 YES shares and sell them at 0.60 bps, you receive 60 USDC.
- The buyer of your YES shares effectively "sold" a NO position at 0.40.

---

## The Decentralized Limit Order Book (DLOB)

Unlike a traditional exchange, there is no on-chain order book data structure. Orders are stored as **individual PDA accounts**. The order book exists only in the memory of keeper bots, reconstructed by querying all open `Order` accounts for a given market.

### DLOB Structure

```
Market DLOB (in-memory, keeper-side)
├─ bids[]  — sorted DESC by price (best bid = highest price first)
│   ├─ [9200] 50 shares  order PDA: 3kJ…
│   ├─ [8000] 100 shares order PDA: 7mN…
│   └─ [6000] 200 shares order PDA: 9pR…
│
└─ asks[]  — sorted ASC by price (best ask = lowest price first)
    ├─ [5900] 80 shares  order PDA: 2aB…
    ├─ [7000] 60 shares  order PDA: 5cD…
    └─ [8500] 120 shares order PDA: 8eF…
```

Orders at the same price are sorted by `created_at` (FIFO).

### SDK DLOB classes

```typescript
import { DLOB, OrderSubscriber } from "@polymarket-sol/sdk";

const subscriber = new OrderSubscriber(connection, marketPubkey, PROGRAM_ID);
await subscriber.subscribe();

const dlob = subscriber.getDLOB();
console.log("Best bid:", dlob.bestBid());   // highest bid price
console.log("Best ask:", dlob.bestAsk());   // lowest ask price
```

---

## Spread and Crossing

The **spread** is the gap between the best ask price and the best bid price:

```
spread = best_ask.price - best_bid.price
```

A **crossing** occurs when `best_bid.price >= best_ask.price`. This means a buyer is willing to pay at least as much as a seller demands — a trade can execute.

Example:

```
Best bid: 6200 bps  (buyer willing to pay 0.62)
Best ask: 6000 bps  (seller willing to accept 0.60)
→ bid ≥ ask  →  crossing  →  fill executes at ask price
```

When the keeper bot detects a crossing, it submits a `FillOrder` transaction. The fill executes at the **bid price** (the buyer pays their stated price, the seller receives `bid_price - fill_fee`).

---

## Partial Fills

Orders do not have to fill completely in a single transaction. The program tracks `fill_amount` on each `Order` account:

```
remaining = size - fill_amount
```

The keeper fills `min(bid.remaining, ask.remaining)` in one transaction. An order remains open (its PDA stays alive) until `fill_amount == size`, at which point the program closes the account and returns rent to the user.

---

## Order Nonce and PDA Uniqueness

Each order has a `nonce` (u64) chosen by the client. This makes each Order PDA unique per `(market, user, nonce)` tuple, allowing a user to hold multiple simultaneous resting orders in the same market.

```
Order PDA seeds: [b"order", market_pubkey, user_pubkey, nonce_le_bytes]
```

A typical approach is to use a monotonic counter or timestamp as the nonce. Reusing a nonce for an existing live order will cause `CreateAccount` to fail on-chain (the account already exists).

---

## Next Steps

- [Positions & Tokens](./positions-and-tokens.md)
- [Order Lifecycle](./order-lifecycle.md)
- [Keeper — Overview](../keeper/overview.md)
