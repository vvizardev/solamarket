# Fetching Markets

> Four strategies for discovering and querying markets and events directly from the Solana RPC — no REST API required.

Because every market and event is a first-class on-chain PDA account, all discovery happens via `getProgramAccounts` with `memcmp` filters or direct account loads. There is no centralized endpoint; any RPC node gives you the same answer.

---

## Strategies at a Glance

| Strategy | Best for | RPC calls |
|---|---|---|
| [By question / event label](#1-by-question--event-label) | Single market or event you already know | **0** — pure key derivation |
| [By event](#2-by-event) | All markets inside one event | **1** — load Event PDA |
| [By category](#3-by-category) | Browse by topic (Politics, Crypto, Sports…) | **1** `getProgramAccounts` |
| [All active markets / events](#4-all-active-markets--events) | Full discovery, indexing | **1–2** `getProgramAccounts` |

> **Tip:** Strategies 1 and 2 are the cheapest. Prefer them when you already know what you are looking for.

---

## Memcmp Offset Reference

All filters assume the Borsh layout documented in [Account Structs](../program/accounts.md).

### Market (discriminant `0`)

| Field | Offset | Type | Notes |
|---|---|---|---|
| `discriminant` | 0 | `u8` | Filter value: `0` |
| `resolved` | 169 | `bool` | `0x00` = unresolved (active) |
| `event` | 211 | `[u8; 32]` | `Pubkey::default()` = standalone |
| `primary_category` | 291 | `u8` | See [Categories](../core-concepts/categories.md) |
| `subcategory` | 292 | `u16` LE | Two bytes, little-endian |

### Event (discriminant `3`)

| Field | Offset | Type | Notes |
|---|---|---|---|
| `discriminant` | 0 | `u8` | Filter value: `3` |
| `resolved` | 74 | `bool` | `0x00` = unresolved (active) |
| `primary_category` | 588 | `u8` | |
| `subcategory` | 589 | `u16` LE | Two bytes, little-endian |

---

## 1. By Question / Event Label

**Best for:** Fetching a single known market or event.

This project is **zero-RPC** for direct lookups. The market PDA is fully deterministic from the question text, and the event PDA from the event label. No slug lookup, no API call.

```
question text  ──SHA-256──►  question_hash  ──PDA derivation──►  Market address
event label    ──SHA-256──►  event_id       ──PDA derivation──►  Event address
```

### Market

```typescript
import { createHash } from "crypto";
import { findMarketPda } from "@solamarket/sdk";

const question = "Will BTC be above $100k by end of 2026?";
const questionHash = new Uint8Array(createHash("sha256").update(question).digest());

// No RPC call — pure key derivation
const [marketPda] = findMarketPda(questionHash, PROGRAM_ID);

// One RPC call to fetch the account
const market = await fetchMarket(connection, marketPda);
```

### Event

```typescript
import { createHash } from "crypto";
import { findEventPda, fetchEvent } from "@solamarket/sdk";

const label = "2024 US Presidential Election";
const eventId = new Uint8Array(createHash("sha256").update(label).digest());

const [eventPda] = findEventPda(eventId, PROGRAM_ID);
const event = await fetchEvent(connection, eventPda);

console.log("Markets:", event.markets.slice(0, event.marketCount));
```

> **Comparison with Polymarket:** Polymarket resolves slugs via `GET /events?slug=fed-decision-in-october` through their [Gamma API](https://docs.polymarket.com/market-data/fetching-markets#fetch-by-slug). Here there is no API server — the address is derived locally and loaded directly from the RPC.

---

## 2. By Event

**Best for:** Loading all markets that belong to one event.

Because the Event PDA stores `markets: [[u8;32]; 16]` inline, one account load gives you every child market address. You can then batch-fetch those market accounts in a single `getMultipleAccounts` call.

```typescript
import { createHash } from "crypto";
import { findEventPda, fetchEvent, deserializeMarket } from "@solamarket/sdk";

const label = "2026 FIFA World Cup Winner";
const eventId = new Uint8Array(createHash("sha256").update(label).digest());
const [eventPda] = findEventPda(eventId, PROGRAM_ID);

// 1 RPC call — load the event
const event = await fetchEvent(connection, eventPda);
const marketAddresses = event.markets.slice(0, event.marketCount);

// 1 RPC call — batch-load all child markets
const accountInfos = await connection.getMultipleAccountsInfo(marketAddresses);
const markets = accountInfos.map(info => deserializeMarket(Buffer.from(info!.data)));
```

**Alternative — filter from the market side** (useful when you don't know the event label but you have the event pubkey):

```typescript
// All Market accounts that reference a specific event (offset 211)
const eventMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } }, // Market discriminant
    { memcmp: { offset: 211, bytes: eventPda.toBase58() } },           // market.event field
  ],
});
```

> This is the on-chain equivalent of Polymarket's `/events` endpoint which embeds associated markets in each event response.

---

## 3. By Category

**Best for:** Browse UI, filtering by topic (Politics, Crypto horizons, Sports…).

All category ids are defined in [Market Categories](../core-concepts/categories.md).

### Markets by primary category

```typescript
const POLITICS = 1;
const CRYPTO   = 3;
const SPORTS   = 2;

const politicsMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },       // Market discriminant
    { memcmp: { offset: 291, bytes: bs58.encode(Buffer.from([POLITICS])) } }, // primary_category
  ],
});
```

### Events by primary category

```typescript
const weatherEvents = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([3])) } },      // Event discriminant
    { memcmp: { offset: 588, bytes: bs58.encode(Buffer.from([4])) } },      // primary_category = Weather
  ],
});
```

### Markets by subcategory (two-byte LE)

`subcategory` is a `u16` stored little-endian at offset 292 (market) or 589 (event).

```typescript
// Crypto markets, 1-hour horizon (primary=3, subcategory=3)
const subBytes = Buffer.alloc(2);
subBytes.writeUInt16LE(3); // subcategory value

const hourlyMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } }, // Market
    { memcmp: { offset: 291, bytes: bs58.encode(Buffer.from([3])) } }, // primary = Crypto
    { memcmp: { offset: 292, bytes: bs58.encode(subBytes) } },         // subcategory = 3 (1-hour)
  ],
});
```

### Category quick reference

| Goal | Account | Discriminant offset | Category offset | Subcategory offset |
|---|---|---|---|---|
| All Politics markets | Market | `0` → `0x00` | `291` | `292` (u16 LE) |
| All Crypto markets | Market | `0` → `0x00` | `291` → `0x03` | `292` |
| All Weather events | Event | `0` → `0x03` | `588` → `0x04` | `589` |
| All Sports events | Event | `0` → `0x03` | `588` → `0x02` | `589` |

> **Comparison with Polymarket:** Polymarket uses `tag_id` query parameters against `GET /events?tag_id=...`. Here the same filter lives on-chain in fixed byte offsets; no API server needed.

---

## 4. All Active Markets / Events

**Best for:** Full market discovery, indexing, analytics.

### All active (unresolved) markets

```typescript
// resolved=false is encoded as 0x00 at offset 169
const activeMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },    // Market discriminant
    { memcmp: { offset: 169, bytes: bs58.encode(Buffer.from([0])) } },    // resolved = false
  ],
});
```

### All resolved markets (historical)

```typescript
const resolvedMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },    // Market discriminant
    { memcmp: { offset: 169, bytes: bs58.encode(Buffer.from([1])) } },    // resolved = true
  ],
});
```

### All events

```typescript
const allEvents = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: bs58.encode(Buffer.from([3])) } },      // Event discriminant
  ],
});
```

### All active events

```typescript
// Event.resolved is at offset 74
const activeEvents = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,  bytes: bs58.encode(Buffer.from([3])) } },     // Event discriminant
    { memcmp: { offset: 74, bytes: bs58.encode(Buffer.from([0])) } },     // resolved = false
  ],
});
```

---

## Pagination

`getProgramAccounts` returns all matching accounts in one response — there is no server-side `limit`/`offset` parameter in the base RPC. Handle large result sets client-side:

```typescript
const PAGE_SIZE = 100;

const allMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },
    { memcmp: { offset: 169, bytes: bs58.encode(Buffer.from([0])) } },
  ],
});

// Client-side pagination
function getPage<T>(items: T[], page: number, size = PAGE_SIZE): T[] {
  return items.slice(page * size, (page + 1) * size);
}

const page1 = getPage(allMarkets, 0); // first 100
const page2 = getPage(allMarkets, 1); // next 100
```

> Some RPC providers (Helius, Triton) offer `getProgramAccountsWithFilters` extensions that support server-side pagination. Check your provider's docs.

### Reducing data transfer with `dataSlice`

When building a list view that only needs a few fields (e.g. `end_time`, `resolved`, `primary_category`), fetch only the bytes you need:

```typescript
// Fetch only bytes 161–295: end_time onward (saves ~161 bytes per market)
const slim = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: bs58.encode(Buffer.from([0])) } },
  ],
  dataSlice: { offset: 0, length: 295 }, // still full for small accounts
});
```

For `Order` accounts (107 bytes) or `UserPosition` (1131 bytes), `dataSlice` saves meaningful bandwidth.

---

## Sorting

`getProgramAccounts` returns accounts in an arbitrary order. Sort client-side after deserialization:

```typescript
import { deserializeMarket } from "@solamarket/sdk";

const markets = allMarkets
  .map(({ account }) => deserializeMarket(Buffer.from(account.data)))
  .filter(m => !m.resolved)                              // active only
  .sort((a, b) => Number(a.endTime - b.endTime));        // soonest expiry first
```

Common sort keys:

| Sort | Field | Type |
|---|---|---|
| Soonest expiry | `end_time` | `i64` |
| Most orders | `order_count` | `u64` |
| By category | `primary_category` → `subcategory` | `u8`, `u16` |

---

## Combining Filters

Filters are ANDed. Add as many `memcmp` entries as needed:

```typescript
// Active Crypto markets with 1-hour horizon subcategory
const subBytes = Buffer.alloc(2);
subBytes.writeUInt16LE(3);

const filtered = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },  // Market
    { memcmp: { offset: 169, bytes: bs58.encode(Buffer.from([0])) } },  // active
    { memcmp: { offset: 291, bytes: bs58.encode(Buffer.from([3])) } },  // Crypto
    { memcmp: { offset: 292, bytes: bs58.encode(subBytes) } },          // 1-hour
  ],
});
```

---

## Standalone vs Event Markets

A market with `market.event == Pubkey::default()` (all zeros) is a standalone market — not grouped under any event. Filter for each type:

```typescript
import { PublicKey } from "@solana/web3.js";

const DEFAULT_PUBKEY = new PublicKey("11111111111111111111111111111111");

// Standalone markets only (event field = all zeros)
const standaloneMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: bs58.encode(Buffer.from([0])) } },
    { memcmp: { offset: 211, bytes: DEFAULT_PUBKEY.toBase58() } },
  ],
});

// Event-linked markets only (event field != all zeros)
// Note: no "not-equal" filter in getProgramAccounts — fetch all and exclude client-side
const allMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [{ memcmp: { offset: 0, bytes: bs58.encode(Buffer.from([0])) } }],
});
const eventLinked = allMarkets.filter(({ account }) => {
  const eventBytes = account.data.slice(211, 243);
  return !eventBytes.every(b => b === 0);
});
```

---

## Best Practices

1. **Know the question text? Derive the PDA.** Use `findMarketPda` / `findEventPda` — zero RPC calls, instant.
2. **Need all markets in an event? Load the Event PDA first.** One account load gives you all child addresses; then `getMultipleAccountsInfo` to batch-fetch the markets.
3. **Building a browse UI? Stack filters.** Combine `discriminant` + `resolved` + `primary_category` to minimize data returned by the RPC.
4. **Skip deserialization when scanning.** Use `dataSlice` if your indexer only needs a subset of fields.
5. **Always filter by discriminant first.** Without it, `getProgramAccounts` returns every account owned by the program, including Orders and UserPositions.
6. **Sort and paginate client-side.** `getProgramAccounts` has no server-side `ORDER BY` or `LIMIT`. Deserialize, sort, then slice.

---

## Comparison with Polymarket

| | [Polymarket](https://docs.polymarket.com/market-data/fetching-markets) | This project |
|---|---|---|
| By specific market | `GET /markets?slug=…` (REST API) | Derive PDA from question hash — zero RPC |
| By category | `GET /events?tag_id=…` | `getProgramAccounts` memcmp at offset 291 (market) or 588 (event) |
| All active markets | `GET /events?active=true&closed=false` | `getProgramAccounts` with discriminant + `resolved=0x00` filter |
| Pagination | `limit` / `offset` query params (server-side) | Client-side slice after full fetch |
| Source of truth | Polymarket's centralized Gamma API | Any Solana RPC node |
| Censorship resistance | API can be taken down or rate-limited | Permissionless — filters run on-chain data |

---

## Next Steps

- [Account Structs](../program/accounts.md) — full Borsh layout and byte offsets
- [Market Categories](../core-concepts/categories.md) — all `primary_category` / `subcategory` ids
- [PDA Seeds](../program/pda-seeds.md) — seed derivation for all account types
- [SDK — Quickstart](../sdk/quickstart.md) — `fetchMarket`, `fetchEvent`, `deserializeMarket` helpers
