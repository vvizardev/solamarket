# Account Structs

> Borsh layout for Market, Order, and UserPosition — byte-exact reference for both Rust and TypeScript.

---

## Discriminants

Each account type has a fixed 1-byte discriminant at offset 0, used to filter accounts with `getProgramAccounts`:

| Account | Discriminant |
|---------|-------------|
| `Market` | `0` |
| `Order` | `1` |
| `UserPosition` | `2` |
| `Event` | `3` |

---

## Market (292 bytes) {#market-292-bytes}

Fee fields support Polymarket-style **taker curve**, **maker fee**, **maker rebate**, and **treasury** routing. See [SDK — Fees](../sdk/fees.md).

```rust
pub struct Market {
    pub discriminant:     u8,       // offset 0
    pub question_hash:    [u8; 32], // offset 1
    pub vault:            Pubkey,   // offset 33
    pub collateral_mint:  Pubkey,   // offset 65
    pub yes_mint:         Pubkey,   // offset 97
    pub no_mint:          Pubkey,   // offset 129
    pub end_time:         i64,      // offset 161
    pub resolved:         bool,     // offset 169
    pub winning_outcome:  u8,       // offset 170
    pub admin:            Pubkey,   // offset 171
    pub order_count:      u64,      // offset 203
    pub event:            Pubkey,   // offset 211  Pubkey::default() = standalone market

    // Fee schedule (0/1 curve numer or denom disables taker curve fee)
    pub taker_curve_numer:           u32,    // offset 243
    pub taker_curve_denom:           u32,    // offset 247
    pub maker_fee_bps:               u16,    // offset 251
    pub maker_rebate_of_taker_bps:   u16,    // offset 253
    pub keeper_reward_of_taker_bps:  u16,    // offset 255
    pub _fee_padding:                u16,    // offset 257  reserved / alignment
    pub fee_recipient_user:          Pubkey, // offset 259  treasury owner; default → keeper absorbs treasury_share

    pub bump:                        u8,     // offset 291
}
// Total: 292 bytes
```

`event` is `Pubkey::default()` (all zeros) for standalone markets. When a market belongs to a multi-market event, this field is set to the event's PDA pubkey via `AddMarketToEvent`. This allows a single `getProgramAccounts` memcmp filter at offset 211 to retrieve all markets in an event without loading the Event account first.

TypeScript equivalent:

```typescript
interface Market {
  discriminant:    number;
  questionHash:    Uint8Array;   // 32 bytes
  vault:           PublicKey;
  collateralMint:  PublicKey;
  yesMint:         PublicKey;    // Pubkey.default() until TokenizePosition
  noMint:          PublicKey;
  endTime:         bigint;       // i64 (little-endian)
  resolved:        boolean;
  winningOutcome:  number;       // 0=unresolved, 1=YES, 2=NO
  admin:           PublicKey;
  orderCount:      bigint;
  event:           PublicKey;    // Pubkey.default() = standalone

  takerCurveNumer:          number;    // u32 LE
  takerCurveDenom:          number;    // u32 LE
  makerFeeBps:              number;    // u16 — bps of fill_cost
  makerRebateOfTakerBps:    number;    // u16 — share of taker_fee
  keeperRewardOfTakerBps:   number;    // u16 — share of taker_fee
  feePadding:               number;    // u16 reserved
  feeRecipientUser:         PublicKey; // treasury; default pubkey → merge treasury into keeper

  bump:            number;
}
```

**Suggested defaults** for a Polymarket-like mainnet launch: `takerCurveNumer = 1`, `takerCurveDenom = 100`, `makerFeeBps = 0`, `makerRebateOfTakerBps = 7000` (example), `keeperRewardOfTakerBps = 500`, `feeRecipientUser = <treasury wallet>`, with `makerRebate + keeperReward ≤ 10_000`.

**Legacy migration:** To approximate the old flat **5 bps from the ask** model, set `takerCurveNumer = 0`, `keeperRewardOfTakerBps = 10_000`, `makerRebateOfTakerBps = 0`, and implement the fallback `keeper_reward = fill_size × 5 / 10_000` when the curve is disabled (exact policy lives in the program).

Deserialize:

```typescript
import { fetchMarket, deserializeMarket } from "@polymarket-sol/sdk";

// Fetches from RPC and deserializes
const market = await fetchMarket(connection, marketPda);

// Or deserialize raw bytes
const market = deserializeMarket(Buffer.from(accountInfo.data));
```

---

## Order (107 bytes)

```rust
pub struct Order {
    pub discriminant:  u8,      // offset 0  (= 1)
    pub market:        Pubkey,  // offset 1
    pub user:          Pubkey,  // offset 33
    pub side:          u8,      // offset 65 (0=bid, 1=ask)
    pub price:         u64,     // offset 66
    pub size:          u64,     // offset 74
    pub fill_amount:   u64,     // offset 82
    pub nonce:         u64,     // offset 90
    pub created_at:    i64,     // offset 98
    pub bump:          u8,      // offset 106
}
// Total: 107 bytes
```

TypeScript equivalent:

```typescript
interface Order {
  discriminant: number;
  market:       PublicKey;
  user:         PublicKey;
  side:         0 | 1;          // 0=bid, 1=ask
  price:        bigint;         // basis points 1–9999
  size:         bigint;         // original size
  fillAmount:   bigint;         // total filled
  nonce:        bigint;
  createdAt:    bigint;         // i64 Unix timestamp
  bump:         number;
}
```

Derived values:

```typescript
const remaining    = order.size - order.fillAmount;
const isFullyFilled = remaining === 0n;
const pricePct     = Number(order.price) / 100;   // e.g. 6000 → 60.00%
```

---

## UserPosition (1131 bytes)

```rust
pub struct UserPosition {
    pub discriminant:      u8,           // offset 0    (= 2)
    pub market:            Pubkey,       // offset 1
    pub user:              Pubkey,       // offset 33
    pub yes_balance:       u64,          // offset 65
    pub no_balance:        u64,          // offset 73
    pub locked_yes:        u64,          // offset 81
    pub locked_no:         u64,          // offset 89
    pub locked_collateral: u64,          // offset 97
    pub open_orders:       [Pubkey; 32], // offset 105  (32 × 32 = 1024 bytes)
    pub open_order_count:  u8,           // offset 1129
    pub bump:              u8,           // offset 1130
}
// Total: 1131 bytes
```

TypeScript equivalent:

```typescript
interface UserPosition {
  discriminant:      number;
  market:            PublicKey;
  user:              PublicKey;
  yesBalance:        bigint;
  noBalance:         bigint;
  lockedYes:         bigint;   // YES locked in open ask orders
  lockedNo:          bigint;   // reserved for future NO-side orders
  lockedCollateral:  bigint;   // USDC locked in open bid orders
  openOrders:        PublicKey[]; // sliced to openOrderCount
  openOrderCount:    number;
  bump:              number;
}
```

Fetch:

```typescript
import { fetchUserPosition, findUserPositionPda } from "@polymarket-sol/sdk";

const [posPda] = findUserPositionPda(marketPda, userPubkey, PROGRAM_ID);
const position = await fetchUserPosition(connection, posPda);

console.log("YES:", Number(position.yesBalance) / 1e6, "USDC");
console.log("Open orders:", position.openOrderCount);
```

---

## Event (589 bytes)

```rust
pub struct Event {
    pub discriminant:  u8,            // offset 0    = 3
    pub event_id:      [u8; 32],      // offset 1    SHA-256(event label string)
    pub admin:         Pubkey,        // offset 33
    pub end_time:      i64,           // offset 65   shared end time for all child markets
    pub is_exclusive:  bool,          // offset 73   if true → ResolveEvent forces non-winners to NO
    pub resolved:      bool,          // offset 74
    pub market_count:  u8,            // offset 75   filled slots (max 16)
    pub markets:       [Pubkey; 16],  // offset 76   16 × 32 = 512 bytes
    pub bump:          u8,            // offset 588
}
// Total: 589 bytes
```

`markets` slots beyond `market_count` contain `Pubkey::default()` and are ignored. The fixed 16-slot array keeps the account size constant at 589 bytes regardless of how many markets have been attached.

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
  markets:       PublicKey[];  // sliced to marketCount; rest are Pubkey.default()
  bump:          number;
}
```

Fetch:

```typescript
import { fetchEvent, findEventPda } from "@polymarket-sol/sdk";
import { createHash } from "crypto";

const eventId = new Uint8Array(createHash("sha256").update("2024 US Presidential Election").digest());
const [eventPda] = findEventPda(eventId, PROGRAM_ID);
const event = await fetchEvent(connection, eventPda);

console.log("Markets:", event.markets.slice(0, event.marketCount).map(p => p.toBase58()));
```

---

## getProgramAccounts Filters

To fetch all accounts of a given type for a specific market or event:

```typescript
// All Order accounts for a market
const orders = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: Buffer.from([1]).toString("base64") } }, // Order discriminant
    { memcmp: { offset: 1, bytes: marketPda.toBase58() } },                // market at offset 1
  ],
});

// All UserPosition accounts for a market
const positions = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: Buffer.from([2]).toString("base64") } }, // UserPosition discriminant
    { memcmp: { offset: 1, bytes: marketPda.toBase58() } },
  ],
});

// All Market accounts belonging to a specific event (uses Market.event at offset 211)
const eventMarkets = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0,   bytes: Buffer.from([0]).toString("base64") } }, // Market discriminant
    { memcmp: { offset: 211, bytes: eventPda.toBase58() } },                 // event field
  ],
});

// All Event accounts (to list all events)
const allEvents = await connection.getProgramAccounts(PROGRAM_ID, {
  filters: [
    { memcmp: { offset: 0, bytes: Buffer.from([3]).toString("base64") } }, // Event discriminant
  ],
});
```

---

## Next Steps

- [PDA Seeds](./pda-seeds.md)
- [Instructions Reference](./instructions.md)
