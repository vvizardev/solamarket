# Orders

> Build, place, cancel, and query orders using the SDK.

---

## Order Types

All orders are **limit orders**. There are no market orders, FOK, FAK, or GTD variants in this implementation — orders rest until filled or manually cancelled.

| Field | Type | Description |
|-------|------|-------------|
| `side` | `OrderSide` | `Bid (0)` = buy YES, `Ask (1)` = sell YES |
| `price` | `bigint` | Limit price in basis points (1–9999) |
| `size` | `bigint` | Collateral units (USDC × 10^6) |
| `nonce` | `bigint` | Client-chosen unique ID; determines Order PDA address |

---

## Instruction Builders

All builders return a `TransactionInstruction`. Add them to a `Transaction` and send with `sendAndConfirmTransaction`.

### `splitInstruction` — Deposit USDC

Deposits USDC into the market vault and credits both `yes_balance` and `no_balance`.

```typescript
splitInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,          // USDC units (6 decimals)
  programId?:     PublicKey,       // defaults to PROGRAM_ID
): TransactionInstruction
```

### `mergeInstruction` — Withdraw USDC

Burns equal YES + NO balance, returns USDC from vault.

```typescript
mergeInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId?:     PublicKey,
): TransactionInstruction
```

### `placeOrderInstruction` — Place a Limit Order

Creates the Order PDA and locks the appropriate balance.

```typescript
placeOrderInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  orderPda:       PublicKey,       // derived via findOrderPda()
  args: {
    side:  OrderSide,
    price: bigint,                 // basis points 1–9999
    size:  bigint,
    nonce: bigint,
  },
  programId?:     PublicKey,
): TransactionInstruction
```

### `cancelOrderInstruction` — Cancel an Order

Closes the Order PDA and releases locked balance. Only the order owner can call this.

```typescript
cancelOrderInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  orderPda:       PublicKey,
  args: { nonce: bigint },
  programId?:     PublicKey,
): TransactionInstruction
```

### `fillOrderInstruction` — Fill Two Crossing Orders (keeper)

```typescript
fillOrderInstruction(
  keeper:          PublicKey,
  marketPda:       PublicKey,
  bidOrderPda:     PublicKey,
  askOrderPda:     PublicKey,
  bidPositionPda:  PublicKey,
  askPositionPda:  PublicKey,
  keeperPositionPda: PublicKey,
  args: { fillSize: bigint },
  programId?:      PublicKey,
): TransactionInstruction
```

### `redeemInstruction` — Redeem Winning Tokens (post-resolution)

```typescript
redeemInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId?:     PublicKey,
): TransactionInstruction
```

---

## Querying Orders

### Fetch a single order

```typescript
import { fetchOrder } from "@solamarket/sdk";

const order = await fetchOrder(connection, orderPda);
console.log("Side:",    order.side === 0 ? "bid" : "ask");
console.log("Price:",   Number(order.price) / 10_000);
console.log("Size:",    Number(order.size) / 1e6, "USDC");
console.log("Filled:",  Number(order.fillAmount) / 1e6, "USDC");
console.log("Remaining:", Number(order.size - order.fillAmount) / 1e6, "USDC");
```

### Fetch all orders for a market

Uses `getProgramAccounts` with two `memcmp` filters: Order discriminant byte (`1`) and market pubkey at byte offset 1.

```typescript
import { fetchOrdersForMarket } from "@solamarket/sdk";

const orders = await fetchOrdersForMarket(connection, marketPda, PROGRAM_ID);

// Sort to reconstruct the DLOB
const bids = orders
  .filter(({ order }) => order.side === 0)
  .sort((a, b) => Number(b.order.price - a.order.price)); // DESC

const asks = orders
  .filter(({ order }) => order.side === 1)
  .sort((a, b) => Number(a.order.price - b.order.price)); // ASC
```

---

## Order Object Fields

```typescript
interface Order {
  discriminant: number;       // always 1
  market:       PublicKey;
  user:         PublicKey;
  side:         0 | 1;        // 0 = bid, 1 = ask
  price:        bigint;       // basis points 1–9999
  size:         bigint;       // original size in USDC units
  fillAmount:   bigint;       // total filled so far
  nonce:        bigint;
  createdAt:    bigint;       // Unix timestamp (i64)
  bump:         number;
}
```

Derived helper:

```typescript
const remaining = order.size - order.fillAmount;
const isFullyFilled = remaining === 0n;
```

---

## Deriving PDAs

```typescript
import {
  findOrderPda,
  findUserPositionPda,
  findMarketPda,
  findVaultAuthorityPda,
} from "@solamarket/sdk";

const [marketPda]   = findMarketPda(questionHash, PROGRAM_ID);
const [orderPda]    = findOrderPda(marketPda, user.publicKey, nonce, PROGRAM_ID);
const [posPda]      = findUserPositionPda(marketPda, user.publicKey, PROGRAM_ID);
const [vaultAuth]   = findVaultAuthorityPda(marketPda, PROGRAM_ID);
```

---

## Error Handling

On-chain errors surface as `SendTransactionError` with a custom program error code. Map them to `PredictionMarketError` variants:

```typescript
try {
  await sendAndConfirmTransaction(connection, tx, [user]);
} catch (err) {
  if (err instanceof SendTransactionError) {
    const logs = err.logs ?? [];
    // Look for: "Program log: AnchorError..." or custom error code
    console.error("Transaction failed:", logs);
  }
}
```

See [Error Codes](../resources/error-codes.md) for the full list.

---

## Next Steps

- [Fees](./fees.md)
- [Outcome Tokens — Split / Merge / Tokenize](./outcome-tokens.md)
- [WebSocket](./websocket.md)
