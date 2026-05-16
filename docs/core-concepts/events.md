# Events

> Grouping multiple markets under a shared event — single-market and multi-market patterns, the Event account, and atomic exclusive resolution.

---

## Overview

An **Event** is an on-chain account that groups one or more binary `Market` accounts under a shared label, admin, and end time. It is the Solana-native equivalent of Polymarket's event layer.

| Pattern | Description | Example |
|---------|-------------|---------|
| **Single-market event** | One binary question; no Event account needed | "Will BTC be above $100k by end of 2025?" |
| **Multi-market event** | Two or more related markets grouped under one Event | "2024 US Presidential Election" → Trump YES/NO, Biden YES/NO, Other YES/NO |

Single-market events use the existing `CreateMarket` instruction directly — no Event account is required. Multi-market events create an `Event` PDA first, then attach markets to it.

---

## Categories

Group-level taxonomy lives on **`Event`** (`primary_category`, `subcategory`). Child **`Market`** accounts should normally duplicate the same ids so clients can filter markets without loading the event account; see [Market Categories](./categories.md).

---

## Event Account

Discriminant `3`. PDA seeds: `[b"event", event_id: [u8; 32]]`. Canonical layout: [Program — Accounts](../program/accounts.md#event-account).

```rust
// state/event.rs
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Event {
    pub discriminant:  u8,            // offset 0    = 3
    pub event_id:      [u8; 32],      // offset 1    SHA-256(event label string)
    pub admin:         Pubkey,        // offset 33
    pub end_time:      i64,           // offset 65   shared deadline for all child markets
    pub is_exclusive:  bool,          // offset 73   if true → ResolveEvent NO's all non-winners atomically
    pub resolved:      bool,          // offset 74
    pub market_count:  u8,            // offset 75   number of slots currently filled (max 16)
    pub markets:       [Pubkey; 16],  // offset 76   16 × 32 = 512 bytes
    pub primary_category: u8,         // offset 588  0 = uncategorized
    pub subcategory:      u16,        // offset 589  meaning depends on primary_category
    pub bump:               u8,       // offset 591
}
// Total: 592 bytes
```

TypeScript equivalent:

```typescript
interface Event {
  discriminant:  number;
  eventId:       Uint8Array;   // 32 bytes
  admin:         PublicKey;
  endTime:       bigint;       // i64 (little-endian)
  isExclusive:   boolean;
  resolved:      boolean;
  marketCount:   number;
  markets:       PublicKey[];  // sliced to marketCount
  primaryCategory: number;
  subcategory:     number;
  bump:          number;
}
```

The `markets` array holds up to **16** market pubkeys. Slots beyond `market_count` contain `Pubkey::default()` and are ignored.

---

## Market → Event Linkage

The `Market` struct has an `event` field (offset 211, 32 bytes). It is `Pubkey::default()` for standalone markets and set to the event's PDA pubkey via `AddMarketToEvent`.

```rust
pub struct Market {
    // ... existing fields ...
    pub event: Pubkey,  // Pubkey::default() = standalone; otherwise = event PDA
    pub primary_category: u8,
    pub subcategory:      u16,
    pub bump:             u8,
}
```

This enables efficient off-chain querying: a `getProgramAccounts` `memcmp` filter at offset 211 returns all markets belonging to a specific event without iterating the Event account.

---

## How to Create a Multi-Market Event

### Step 1 — Create the Event

```typescript
import { createHash } from "crypto";
import { createEvent, findEventPda } from "@solamarket/sdk";

const label = "2024 US Presidential Election";
const eventId = new Uint8Array(createHash("sha256").update(label).digest());
const endTime  = BigInt(Math.floor(new Date("2024-11-05T23:59:59Z").getTime() / 1000));

const [eventPda] = findEventPda(eventId, PROGRAM_ID);

const ix = createEvent({
  eventId,
  endTime,
  isExclusive: true,   // exactly one market will resolve YES; all others resolve NO
  admin: adminKeypair.publicKey,
  programId: PROGRAM_ID,
});
```

Wire **`primary_category`** and **`subcategory`** in the `CreateEvent` instruction arguments when the program/SDK exposes them ([Instructions — CreateEvent](../program/instructions.md#9---createevent), [Market Categories](./categories.md)).

### Step 2 — Create each Market

Each market is created independently with `CreateMarket`. The `end_time` passed to each market should match the event's `end_time`.

```typescript
import { createMarket, findMarketPda } from "@solamarket/sdk";
import { createHash } from "crypto";

const questions = [
  "Will Trump win the 2024 US Presidential Election?",
  "Will Biden win the 2024 US Presidential Election?",
  "Will a third-party candidate win the 2024 US Presidential Election?",
];

const marketPdas = questions.map((q) => {
  const hash = new Uint8Array(createHash("sha256").update(q).digest());
  const [pda] = findMarketPda(hash, PROGRAM_ID);
  return { hash, pda };
});
```

Each **`CreateMarket`** call should use **matching** `primary_category` / `subcategory` values so memcmp filters work on `Market` accounts ([Market Categories](./categories.md)).

### Step 3 — Attach Markets to Event

Call `AddMarketToEvent` once per market. This sets `market.event = eventPda` and appends the market pubkey to `event.markets[]`.

```typescript
import { addMarketToEvent } from "@solamarket/sdk";

for (const { pda: marketPda } of marketPdas) {
  const ix = addMarketToEvent({
    admin:     adminKeypair.publicKey,
    eventPda,
    marketPda,
    programId: PROGRAM_ID,
  });
  await sendAndConfirmTransaction(connection, new Transaction().add(ix), [adminKeypair]);
}
```

---

## Resolution

### Non-exclusive events

Resolve each market independently using `ResolveMarket` as usual. The event itself is informational.

### Exclusive events (`is_exclusive = true`)

Use `ResolveEvent` (instruction 11) to resolve the entire event atomically in a single transaction:

```typescript
import { resolveEvent } from "@solamarket/sdk";

// winningIndex = index into event.markets[] that should resolve YES
const ix = resolveEvent({
  admin:         adminKeypair.publicKey,
  eventPda,
  marketPdas:    marketPdas.map(m => m.pda),  // must match event.markets[] order
  winningIndex:  0,                            // Trump wins → markets[0] → YES
  programId:     PROGRAM_ID,
});
```

`ResolveEvent` performs the following atomically:
- Sets `event.resolved = true`.
- Calls the equivalent of `ResolveMarket(YES)` on `markets[winningIndex]`.
- Calls the equivalent of `ResolveMarket(NO)` on all other markets in the event.

All N markets resolve in one transaction — no risk of partial resolution.

---

## Querying Events Off-Chain

### Fetch an event by label

```typescript
import { createHash } from "crypto";
import { findEventPda, fetchEvent } from "@solamarket/sdk";

const label   = "2024 US Presidential Election";
const eventId = new Uint8Array(createHash("sha256").update(label).digest());
const [pda]   = findEventPda(eventId, PROGRAM_ID);

const event = await fetchEvent(connection, pda);
console.log("Markets:", event.markets.slice(0, event.marketCount));
```

### Fetch all markets in an event (memcmp filter)

```typescript
// Market.event is at offset 211 (32 bytes)
const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: Buffer.from([0]).toString("base64") } }, // Market discriminant
    { memcmp: { offset: 211, bytes: eventPda.toBase58() } },                 // event field
  ],
});
```

This is the most efficient query path — one RPC call returns all markets in an event.

### Fetch all Event accounts

```typescript
const eventAccounts = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: Buffer.from([3]).toString("base64") } }, // Event discriminant
  ],
});
```

---

## Exclusive vs. Non-Exclusive Events

| | Exclusive | Non-Exclusive |
|---|---|---|
| Use case | Presidential race, sports tournament (single winner) | Multiple independent yes/no markets with a shared theme |
| Resolution | `ResolveEvent` (atomic, single tx) | `ResolveMarket` per market (individual) |
| Constraint | Exactly one YES outcome; rest forced to NO | No constraint — markets resolve independently |
| Example | "Who wins the 2024 election?" | "Which NBA teams make the playoffs?" |

---

## Constraints Summary

| Rule | Enforced by |
|------|------------|
| `event.admin` must sign `CreateEvent`, `AddMarketToEvent`, `ResolveEvent` | On-chain signer check |
| `market.admin == event.admin` required before `AddMarketToEvent` | On-chain check → `EventAdminMismatch` |
| Max 16 markets per event | On-chain check → `EventFull` |
| A market can only belong to one event | On-chain check → `MarketAlreadyInEvent` |
| `ResolveEvent` only callable on unresolved events | On-chain check → `EventAlreadyResolved` |
| `winningIndex` must be < `event.market_count` | On-chain check → `InvalidMarketIndex` |

---

## Next Steps

- [Market Categories](./categories.md)
- [Instructions — CreateEvent / AddMarketToEvent / ResolveEvent](../program/instructions.md#9---createevent)
- [Account Structs — Event](../program/accounts.md#event-account)
- [PDA Seeds — Event](../program/pda-seeds.md)
- [Resolution](./resolution.md)
