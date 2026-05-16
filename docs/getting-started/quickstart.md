# Quickstart

> Deploy the program, create a market, and place your first order on Solana Devnet.

---

## Prerequisites

- **Rust** — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Solana CLI** — `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`
- **Node.js ≥ 18** and **pnpm** — `npm install -g pnpm`

---

## Step 1 — Clone and Install

```bash
git clone <repo-url> polymarket-forking-solana
cd polymarket-forking-solana
pnpm install
```

---

## Step 2 — Configure Devnet

```bash
# Point CLI at devnet
solana config set --url devnet

# Create wallets (skip if you have them already)
solana-keygen new -o wallet/admin.json --no-bip39-passphrase
solana-keygen new -o wallet/keeper.json --no-bip39-passphrase

# Fund both wallets with devnet SOL
solana airdrop 2 $(solana-keygen pubkey wallet/admin.json) --url devnet
solana airdrop 1 $(solana-keygen pubkey wallet/keeper.json) --url devnet
```

---

## Step 3 — Deploy Mock USDC and Fund Admin

The market vault uses a mock USDC SPL token created on devnet.

```bash
pnpm ts-node scripts/fund-wallet.ts
```

This script creates the mock USDC mint (or reuses an existing one), mints tokens to the admin wallet, and prints the mint address. Copy it — you'll need it for the next step.

---

## Step 4 — Build and Deploy the Program

```bash
# Build to BPF bytecode
cargo build-sbf --manifest-path program/Cargo.toml

# Deploy (note the program ID printed after deploy)
solana program deploy \
  target/deploy/prediction_market.so \
  --keypair wallet/admin.json \
  --url devnet
```

Update `sdk/src/constants.ts` with the printed program ID:

```typescript
export const PROGRAM_ID = new PublicKey("YOUR_PROGRAM_ID_HERE");
```

---

## Step 5 — Create a Market

```bash
pnpm ts-node scripts/create-market.ts
```

This creates a `Market` PDA on-chain with a sample question hash, initializes the USDC vault ATA, and prints the market public key.

---

## Step 6 — Place Opposing Orders

Use the SDK to place a bid and an ask from two different wallets so the keeper can fill them.

```typescript
import { Connection, Keypair } from "@solana/web3.js";
import {
  findMarketPda,
  findOrderPda,
  findUserPositionPda,
  placeOrderInstruction,
  splitInstruction,
} from "@solamarket/sdk";
import { PROGRAM_ID } from "@solamarket/sdk/constants";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// Assume bidder and asker are loaded Keypairs with SOL and mock USDC
const MARKET_PUBKEY = /* market pubkey from step 5 */;

// --- Bidder: buy 100 YES at price 6000 bps (60 cents) ---
const bidNonce = 1n;
const [bidOrderPda] = findOrderPda(MARKET_PUBKEY, bidder.publicKey, bidNonce, PROGRAM_ID);
const [bidPosPda]   = findUserPositionPda(MARKET_PUBKEY, bidder.publicKey, PROGRAM_ID);

// First, deposit collateral (Split)
const splitIx = splitInstruction(
  bidder.publicKey, MARKET_PUBKEY, bidPosPda,
  bidderUsdcAta, vaultAta, vaultAuthority,
  100_000_000n, // 100 USDC (6 decimals)
  PROGRAM_ID,
);

// Then place the bid
const placeBidIx = placeOrderInstruction(
  bidder.publicKey, MARKET_PUBKEY, bidPosPda, bidOrderPda,
  { side: 0, price: 6000n, size: 100_000_000n, nonce: bidNonce },
  PROGRAM_ID,
);

// --- Asker: sell 100 YES at price 5900 bps (59 cents) ---
// (mirror pattern for asker wallet)
```

---

## Step 7 — Start the Keeper Bot

```bash
# Set environment variables
export RPC_ENDPOINT="https://api.devnet.solana.com"
export WS_ENDPOINT="wss://api.devnet.solana.com"
export PROGRAM_ID="<YOUR_PROGRAM_ID>"
export KEEPER_KEYPAIR="wallet/keeper.json"
export MARKET_PUBKEYS="<MARKET_PUBKEY>"

# Start the keeper
pnpm --filter keeper start
```

Watch the logs — you should see the keeper detect the crossing and submit a `FillOrder` transaction:

```
[Keeper] subscribed to market ABC12345… bids=1 asks=1
[Filler] filled 100000000 — tx: 5kJ3...
```

---

## Step 8 — Resolve and Redeem

```bash
# Resolve the market (admin only)
pnpm ts-node scripts/resolve-market.ts
```

After resolution, the winning side uses `Redeem` to swap their YES or NO balance back for USDC at 1:1.

```typescript
import { redeemInstruction } from "@solamarket/sdk";

const redeemIx = redeemInstruction(
  winner.publicKey, MARKET_PUBKEY, winnerPosPda,
  winnerUsdcAta, vaultAta, vaultAuthority,
  100_000_000n, // amount to redeem
  PROGRAM_ID,
);
```

---

## Next Steps

- [Core Concepts — Order Lifecycle](../core-concepts/order-lifecycle.md) — full state machine detail
- [SDK — Orders](../sdk/orders.md) — all instruction builders with full account lists
- [Keeper — Getting Started](../keeper/getting-started.md) — keeper configuration reference
