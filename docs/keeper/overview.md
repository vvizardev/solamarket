# Keeper Bots — Overview

> What keeper bots are, why they exist, and how they fit into the DLOB architecture.

---

## What Is a Keeper Bot?

A keeper bot is a TypeScript daemon that watches on-chain Order accounts and submits `FillOrder` transactions when two orders cross. There is no central matching engine — keepers collectively provide the matching service.

Anyone can run a keeper. There is no whitelist, no registration, and no admin permission required. The `FillOrder` instruction only checks that the keeper signed the transaction — not who they are.

---

## Why Keepers Instead of an On-Chain Matching Engine?

Solana programs can read and write accounts in a single transaction, but they cannot "loop over all orders and find matches" — the compute budget is too limited, and writing to an unbounded number of accounts in one transaction is not feasible.

The DLOB pattern (from Drift Protocol v2) solves this by:

1. Storing orders as individual on-chain accounts.
2. Having off-chain bots maintain a sorted in-memory order book.
3. Having bots submit fill transactions when they detect crosses.

The on-chain program only validates each individual fill — it does not need to search for matches.

---

## Keeper Responsibilities

| Task | How |
|------|-----|
| Load all open orders for a market | `getProgramAccounts` with `memcmp` filters |
| Build and maintain in-memory DLOB | Insert/update/remove on WebSocket events |
| Detect crosses | `best_bid.price >= best_ask.price` |
| Simulate fills | `connection.simulateTransaction` before sending |
| Submit `FillOrder` tx | `sendAndConfirmTransaction` |
| Handle race conditions | Catch `AccountNotFound`; remove stale order from DLOB |
| Handle partial fills | Update local `fillAmount` on `DLOBNode` after success |

---

## Permissionless Matching

Because matching is permissionless:

- **Censorship resistance** — no single operator can prevent an order from being filled if a crossing counterpart exists.
- **Liveness** — the system keeps working as long as at least one honest keeper is running.
- **Competition** — multiple keepers race for the same fill. The first to land the transaction earns the fill fee.

This mirrors Drift Protocol's permissionless filler design, where anyone can submit fills and collect the fill reward.

---

## What the Keeper Does NOT Do

- Cancel orders (only the order owner can cancel).
- Resolve markets (only the admin can resolve).
- Store orders off-chain (the on-chain accounts are the source of truth).
- Guarantee order execution timing (there is latency between order placement and fill).

---

## Keeper vs. Polymarket Operator

| Dimension | Polymarket Operator | Keeper Bot |
|-----------|--------------------|--------------------|
| Who runs it | Polymarket Inc. (centralized) | Anyone |
| Permission | Required | None (permissionless) |
| Revenue | Transaction fees / spread capture | 5 bps fill fee |
| Order censorship | Possible | Not possible |
| Single point of failure | Yes | No (any keeper can fill) |

---

## Next Steps

- [Getting Started](./getting-started.md) — run the keeper bot
- [Economics](./economics.md) — costs and profitability
- [Operations](./operations.md) — race handling and reliability
