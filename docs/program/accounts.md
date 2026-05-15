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

---

## Market (212 bytes)

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
    pub bump:             u8,       // offset 211
}
// Total: 212 bytes
```

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
  bump:            number;
}
```

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

## getProgramAccounts Filters

To fetch all accounts of a given type for a specific market:

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
```

---

## Next Steps

- [PDA Seeds](./pda-seeds.md)
- [Instructions Reference](./instructions.md)
