# SDK Quickstart

> Connect to devnet, fetch market data, and place your first order.

---

## Setup

```typescript
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  fetchMarket,
  fetchUserPosition,
  findMarketPda,
  findUserPositionPda,
  PROGRAM_ID,
} from "@polymarket-sol/sdk";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
```

---

## Fetch a Market

If you know the question string, derive the market PDA directly:

```typescript
import { createHash } from "crypto";
import { findMarketPda, fetchMarket } from "@polymarket-sol/sdk";

const question = "Will BTC be above $100k by end of 2025?";
const hash = new Uint8Array(createHash("sha256").update(question).digest());

const [marketPda] = findMarketPda(hash, PROGRAM_ID);
const market = await fetchMarket(connection, marketPda);

console.log("Market resolved:", market.resolved);
console.log("End time:", new Date(Number(market.endTime) * 1000).toISOString());
```

If you have the market pubkey directly:

```typescript
const market = await fetchMarket(connection, new PublicKey("ABC123..."));
```

---

## Fetch All Orders for a Market

```typescript
import { fetchOrdersForMarket } from "@polymarket-sol/sdk";

const orders = await fetchOrdersForMarket(connection, marketPda, PROGRAM_ID);

const bids = orders.filter(({ order }) => order.side === 0);
const asks = orders.filter(({ order }) => order.side === 1);

console.log(`Bids: ${bids.length}, Asks: ${asks.length}`);
```

---

## Deposit Collateral (Split)

Before placing a bid order you need `yes_balance` and `no_balance`. Call `Split` to deposit USDC into the vault:

```typescript
import {
  splitInstruction,
  findUserPositionPda,
  findVaultAuthorityPda,
} from "@polymarket-sol/sdk";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { Transaction, sendAndConfirmTransaction } from "@solana/web3.js";

const user: Keypair = /* your loaded keypair */;
const collateralMint = market.collateralMint;

const [userPositionPda] = findUserPositionPda(marketPda, user.publicKey, PROGRAM_ID);
const [vaultAuthority]  = findVaultAuthorityPda(marketPda, PROGRAM_ID);
const userUsdcAta       = getAssociatedTokenAddressSync(collateralMint, user.publicKey);
const vaultAta          = market.vault;

const splitIx = splitInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  userUsdcAta,
  vaultAta,
  vaultAuthority,
  100_000_000n,   // 100 USDC (6 decimals)
  PROGRAM_ID,
);

const tx = new Transaction().add(splitIx);
await sendAndConfirmTransaction(connection, tx, [user]);
```

---

## Place a Bid Order (Buy YES)

```typescript
import {
  placeOrderInstruction,
  findOrderPda,
} from "@polymarket-sol/sdk";
import { OrderSide } from "@polymarket-sol/sdk/types";

const nonce = BigInt(Date.now()); // unique per order
const [orderPda] = findOrderPda(marketPda, user.publicKey, nonce, PROGRAM_ID);

const placeBidIx = placeOrderInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  orderPda,
  {
    side:  OrderSide.Bid,   // 0
    price: 6000n,           // 60 cents (6000 basis points)
    size:  50_000_000n,     // 50 USDC
    nonce,
  },
  PROGRAM_ID,
);

const tx = new Transaction().add(placeBidIx);
const sig = await sendAndConfirmTransaction(connection, tx, [user]);
console.log("Order placed:", sig);
console.log("Order PDA:", orderPda.toBase58());
```

---

## Cancel an Order

```typescript
import { cancelOrderInstruction } from "@polymarket-sol/sdk";

const cancelIx = cancelOrderInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  orderPda,
  { nonce },
  PROGRAM_ID,
);

await sendAndConfirmTransaction(connection, new Transaction().add(cancelIx), [user]);
```

---

## Fetch User Position

```typescript
import { fetchUserPosition } from "@polymarket-sol/sdk";

const position = await fetchUserPosition(connection, userPositionPda);
console.log("YES balance:", Number(position.yesBalance) / 1e6, "USDC");
console.log("NO balance:",  Number(position.noBalance)  / 1e6, "USDC");
console.log("Open orders:", position.openOrderCount);
```

---

## Next Steps

- [Orders — full instruction reference](./orders.md)
- [WebSocket — real-time order book](./websocket.md)
- [Fees](./fees.md)
