# Keeper Operations

> Race handling, WebSocket reliability, monitoring, and devnet configuration.

---

## Race Handling

When multiple keepers are running for the same market, they will race to fill the same crossing orders. The keeper that lands its transaction first wins the fill fee; others receive an error.

The `Filler` handles this gracefully:

```typescript
// From keeper/src/Filler.ts
try {
  const sig = await sendAndConfirmTransaction(connection, tx, [this.keeper]);
  console.log(`[Filler] filled ${fillSize} — tx: ${sig}`);
  bid.applyFill(fillSize);
  ask.applyFill(fillSize);
  return sig;
} catch (err) {
  const msg = err instanceof Error ? err.message : String(err);
  if (msg.includes("AccountNotFound") || msg.includes("custom program error: 0x19")) {
    // Order no longer exists — another keeper already filled it
    console.info("[Filler] order already filled by another keeper");
    // TODO: remove stale nodes from DLOB
  } else {
    console.error("[Filler] fill failed:", msg);
  }
  return null;
}
```

`AccountNotFound` is the most common race outcome: by the time your transaction executes, the Order PDA has already been closed by the winning keeper.

### Simulation Check

Before sending, `Filler.fill()` simulates the transaction:

```typescript
const simulation = await this.connection.simulateTransaction(tx, [this.keeper]);
if (simulation.value.err) {
  console.warn("[Filler] simulation error:", simulation.value.err);
  return null;  // skip — order likely already filled
}
```

Simulation catches most races without spending SOL on a doomed transaction. However, there is a small window between simulation success and actual submission where another keeper can win.

---

## Stale DLOB Entries

After a race loss, the local DLOB may still contain the filled order. The keeper should remove it:

```typescript
subscriber.onUpdate((pubkey, node) => {
  if (node === null) {
    // Account was closed — remove from DLOB
    dlob.remove(pubkey);
  }
});
```

The `OrderSubscriber` receives `null` in the callback when an account is closed, which should trigger removal from the local book.

---

## WebSocket Reliability

Solana's public devnet WebSocket endpoint can drop connections or miss events. The keeper uses two complementary mechanisms:

| Mechanism | Purpose |
|-----------|---------|
| WebSocket `accountSubscribe` | Real-time updates (low latency) |
| Periodic `getProgramAccounts` poll | Fallback for missed events |

Configure the poll interval via `POLL_INTERVAL_MS` (default: 2000ms):

```bash
export POLL_INTERVAL_MS=1000   # more aggressive polling
```

For production-quality keeper operation, use a dedicated RPC provider with reliable WebSocket support:

```bash
# Helius (recommended for devnet testing)
export RPC_ENDPOINT="https://devnet.helius-rpc.com/?api-key=YOUR_KEY"
export WS_ENDPOINT="wss://devnet.helius-rpc.com/?api-key=YOUR_KEY"
```

---

## Monitoring

### Log Output Reference

| Log line | Meaning |
|----------|---------|
| `[Keeper] subscribed to market ABC… bids=3 asks=2` | Startup successful; initial DLOB loaded |
| `[Filler] filled 50000000 — tx: 5kJ3…` | Successful fill; tx hash shown |
| `[Filler] simulation error for bid=… ask=…: …` | Simulation failed; fill skipped |
| `[Filler] order already filled by another keeper` | Race condition; another keeper won |
| `[Filler] fill failed: …` | Unexpected error; check logs |
| `[Keeper] no markets configured` | `MARKET_PUBKEYS` env var is not set |

### Checking keeper position balance

```typescript
import { fetchUserPosition, findUserPositionPda } from "@solamarket/sdk";

const [keeperPosPda] = findUserPositionPda(marketPda, keeper.publicKey, PROGRAM_ID);
const pos = await fetchUserPosition(connection, keeperPosPda);

console.log("Keeper no_balance (fill fees):", Number(pos.noBalance) / 1e6, "USDC");
```

---

## Keeper Lifecycle (Process Management)

For long-running keeper operation, use a process manager:

```bash
# PM2
pm2 start pnpm --name keeper -- --filter keeper start
pm2 logs keeper

# Or systemd (Linux)
# Create a unit file in /etc/systemd/system/keeper.service
```

The keeper process exits with code `1` on fatal errors (config missing, RPC unreachable on startup). It does not auto-restart on runtime errors — use a process manager for that.

---

## Devnet Specifics

- **Airdrop rate limit**: Devnet limits airdrops to 2 SOL per request, max a few times per hour per wallet.
- **Public RPC rate limits**: The public devnet endpoint throttles `getProgramAccounts` calls. If you see 429 errors, switch to a free-tier dedicated RPC.
- **Epoch resets**: Solana devnet occasionally resets all on-chain state. You will need to redeploy the program and recreate markets after a reset.

---

## Next Steps

- [Keeper Economics](./economics.md)
- [SDK — WebSocket](../sdk/websocket.md)
