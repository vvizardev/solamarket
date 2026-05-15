# Keeper — Getting Started

> Configure and run the keeper bot daemon.

---

## Prerequisites

- Node.js ≥ 18 and pnpm installed.
- A funded devnet keeper wallet (`wallet/keeper.json`).
- The program deployed and `PROGRAM_ID` known.
- At least one market created with open orders on both sides.

---

## Keeper Wallet

The keeper wallet pays Solana transaction fees for each `FillOrder` submission. It must hold enough SOL to cover fees.

```bash
# Create keeper wallet
solana-keygen new -o wallet/keeper.json --no-bip39-passphrase

# Fund with devnet SOL
solana airdrop 1 $(solana-keygen pubkey wallet/keeper.json) --url devnet
```

The keeper also needs a `UserPosition` account initialized in each market it wants to fill — this is where fill fees accumulate. Initialize it by calling `Split` once (even with a tiny amount) for that market using the keeper wallet, or have the keeper's `UserPosition` pre-created before starting the bot.

---

## Configuration

The keeper reads all configuration from environment variables. Create a `.env` file or export them in your shell:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROGRAM_ID` | Yes | `111...1` | Deployed program public key |
| `KEEPER_KEYPAIR` | Yes | `../../wallet/keeper.json` | Path to keeper keypair JSON |
| `MARKET_PUBKEYS` | Yes | (none) | Comma-separated market public keys to watch |
| `RPC_ENDPOINT` | No | `https://api.devnet.solana.com` | HTTP RPC endpoint |
| `WS_ENDPOINT` | No | `wss://api.devnet.solana.com` | WebSocket endpoint |
| `POLL_INTERVAL_MS` | No | `2000` | Polling interval fallback (ms) |
| `MIN_FILL_SIZE` | No | `1000` | Minimum fill size in collateral units |

Example `.env`:

```bash
PROGRAM_ID=7XyK...
KEEPER_KEYPAIR=./wallet/keeper.json
MARKET_PUBKEYS=ABCde...,XYZfg...
RPC_ENDPOINT=https://devnet.helius-rpc.com/?api-key=YOUR_KEY
WS_ENDPOINT=wss://devnet.helius-rpc.com/?api-key=YOUR_KEY
POLL_INTERVAL_MS=2000
MIN_FILL_SIZE=1000
```

---

## Running the Bot

```bash
# From project root
pnpm --filter keeper start

# Or directly
cd keeper
pnpm start
```

Expected output:

```
[Keeper] starting — program=7XyK1234 keeper=KpR5abcd markets=2
[Keeper] subscribed to market ABCde123… bids=3 asks=2
[Keeper] subscribed to market XYZfg456… bids=0 asks=0
```

When a crossing is detected and filled:

```
[Filler] filled 50000000 — tx: 5kJ3mNqP...
```

When racing against another keeper:

```
[Filler] order already filled by another keeper
```

---

## Multiple Markets

Set `MARKET_PUBKEYS` as a comma-separated list. The keeper runs independent `OrderSubscriber` instances for each market in parallel:

```bash
export MARKET_PUBKEYS="ABC123...,DEF456...,GHI789..."
```

Each market has its own in-memory DLOB and independent WebSocket subscription.

---

## `minFillSize` — Skipping Dust

The `MIN_FILL_SIZE` config prevents the keeper from submitting fills so small that the SOL transaction fee exceeds the fill fee revenue:

```typescript
// Filler.ts
if (fillSize < this.minFillSize) {
  return null; // skip dust
}
```

Default: `1000` units (0.001 USDC). Adjust based on current SOL price and tx fee levels.

---

## Keeper Bot Architecture

```
keeper/src/index.ts          — main daemon loop
  │
  └─ runMarket(market)
       │
       ├─ OrderSubscriber.subscribe()
       │    ├─ getProgramAccounts (initial snapshot)
       │    └─ WebSocket (real-time updates)
       │
       ├─ onUpdate → tryCross()
       │    └─ DLOB.findCross() → Filler.fill(bid, ask)
       │
       └─ setInterval (poll fallback) → tryCross()

keeper/src/Filler.ts         — builds and submits FillOrder txns
keeper/src/config.ts         — reads env vars into KeeperConfig
```

---

## Stopping the Bot

`Ctrl+C` — the process exits cleanly. WebSocket subscriptions are closed automatically by Node.js process termination.

---

## Next Steps

- [Economics](./economics.md) — fill fee math and profitability
- [Operations](./operations.md) — race handling and RPC reliability
