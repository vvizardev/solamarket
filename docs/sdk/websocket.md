# WebSocket & Real-Time Order Book

> Subscribe to live order changes using `OrderSubscriber`.

---

## Overview

There is no dedicated WebSocket endpoint for this project (unlike Polymarket's managed WebSocket stream). Instead, `OrderSubscriber` uses **Solana's native account-change WebSocket** to receive notifications whenever any `Order` account in a market is created, updated, or closed.

```typescript
import { OrderSubscriber } from "@polymarket-sol/sdk";

const subscriber = new OrderSubscriber(connection, marketPubkey, PROGRAM_ID);
await subscriber.subscribe();
```

---

## How It Works

### 1. Initial Snapshot

On `subscribe()`, the SDK calls `getProgramAccounts` with two `memcmp` filters to fetch all existing Order accounts for the market:

```typescript
// Internal to OrderSubscriber
connection.getProgramAccounts(programId, {
  filters: [
    { memcmp: { offset: 0, bytes: "02" } },                 // discriminant = 1 (Order)
    { memcmp: { offset: 1, bytes: marketPubkey.toBase58() } }, // market at offset 1
  ],
});
```

These accounts are deserialized and inserted into the in-memory `DLOB`.

### 2. WebSocket Subscription

After the snapshot, `OrderSubscriber` opens a Solana `accountSubscribe` WebSocket for each Order account. When an order's data changes (partial fill) or the account is closed (full fill or cancel), the subscriber receives a notification and updates the DLOB accordingly.

Additionally, `programSubscribe` monitors new account creations matching the market's filters, so newly placed orders are picked up in real time.

### 3. Polling Fallback

`OrderSubscriber` also accepts a poll interval (configured in the keeper bot via `POLL_INTERVAL_MS`) as a fallback in case WebSocket events are missed. The keeper bot polls on a timer in addition to reacting to WebSocket events.

---

## `OrderSubscriber` API

```typescript
class OrderSubscriber {
  constructor(
    connection:   Connection,
    marketPubkey: PublicKey,
    programId:    PublicKey,
  )

  // Start subscriptions; fetches initial snapshot
  async subscribe(): Promise<void>

  // Stop all subscriptions and clear state
  async unsubscribe(): Promise<void>

  // Returns the current in-memory DLOB
  getDLOB(): DLOB

  // Register a callback for any DLOB update
  onUpdate(callback: (pubkey: PublicKey, node: DLOBNode | null) => void): void
}
```

---

## DLOB API

```typescript
class DLOB {
  // Best bid price (highest)
  bestBid(): bigint | null

  // Best ask price (lowest)
  bestAsk(): bigint | null

  // Returns [bid, ask] if a crossing exists, otherwise null
  findCross(): [DLOBNode, DLOBNode] | null

  // Number of bids and asks
  bidCount: number
  askCount: number
}
```

---

## DLOBNode

Each entry in the DLOB wraps an `Order` with helpers:

```typescript
class DLOBNode {
  pubkey:  PublicKey
  order:   Order

  get remaining(): bigint   // order.size - order.fillAmount
  applyFill(amount: bigint): void  // updates local fill state after a successful fill
}
```

---

## React Hook (`useOrderBook`)

The Next.js frontend wraps `OrderSubscriber` in a React hook:

```typescript
// app/src/hooks/useOrderBook.ts
import { useOrderBook } from "../hooks/useOrderBook";

function OrderBookPanel({ marketPubkey }: { marketPubkey: PublicKey }) {
  const { bids, asks, loading } = useOrderBook(marketPubkey);

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h3>Asks</h3>
      {asks.map(node => (
        <div key={node.pubkey.toBase58()}>
          {Number(node.order.price) / 10_000} — {Number(node.remaining) / 1e6} USDC
        </div>
      ))}
      <h3>Bids</h3>
      {bids.map(node => (
        <div key={node.pubkey.toBase58()}>
          {Number(node.order.price) / 10_000} — {Number(node.remaining) / 1e6} USDC
        </div>
      ))}
    </div>
  );
}
```

---

## RPC Reliability

The public devnet endpoint (`api.devnet.solana.com`) can be unreliable for WebSocket subscriptions. For production-grade keeper operation, use a dedicated RPC provider:

| Provider | Free tier |
|----------|-----------|
| [Helius](https://helius.dev) | 100k credits/day |
| [Alchemy](https://www.alchemy.com/solana) | 300M compute units/month |
| [QuickNode](https://www.quicknode.com) | Shared endpoint available |

Set the custom endpoints in the keeper config:

```bash
export RPC_ENDPOINT="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
export WS_ENDPOINT="wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

---

## Next Steps

- [Keeper — Overview](../keeper/overview.md)
- [Keeper — Getting Started](../keeper/getting-started.md)
