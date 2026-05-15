# Program & Deployment

> Program ID, deploy commands, devnet configuration, and open questions.

---

## Devnet Configuration

```
Cluster:    Solana Devnet
RPC:        https://api.devnet.solana.com
WebSocket:  wss://api.devnet.solana.com
Admin:      wallet/admin.json
Keeper:     wallet/keeper.json
Program ID: set in sdk/src/constants.ts after first deploy
```

---

## Build and Deploy

```bash
# 1. Build the program to BPF bytecode
cargo build-sbf --manifest-path program/Cargo.toml
# Output: target/deploy/prediction_market.so

# 2. Deploy to devnet
solana program deploy \
  target/deploy/prediction_market.so \
  --keypair wallet/admin.json \
  --url devnet
# Prints: Program Id: <PROGRAM_ID>

# 3. Update the SDK constant
# sdk/src/constants.ts:
#   export const PROGRAM_ID = new PublicKey("<PROGRAM_ID>");
```

---

## Upgrade

Solana programs are upgradeable if the deployer retains the upgrade authority:

```bash
# Upgrade existing deployment (same program ID)
solana program deploy \
  target/deploy/prediction_market.so \
  --keypair wallet/admin.json \
  --program-id <PROGRAM_ID> \
  --url devnet
```

---

## Program ID in SDK

```typescript
// sdk/src/constants.ts
import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "11111111111111111111111111111111" // replace with real program ID after deploy
);

export const FILL_FEE_BPS = 5n;
```

All SDK functions accept an optional `programId` parameter that defaults to this constant. Pass a custom `programId` to use a different deployment (e.g., in tests):

```typescript
const [pda] = findMarketPda(hash, new PublicKey("TestProgram..."));
```

---

## Wallet Setup

```bash
# Create wallets
solana-keygen new -o wallet/admin.json  --no-bip39-passphrase
solana-keygen new -o wallet/keeper.json --no-bip39-passphrase

# Fund with devnet SOL (rate-limited; repeat if needed)
solana airdrop 2 $(solana-keygen pubkey wallet/admin.json)  --url devnet
solana airdrop 1 $(solana-keygen pubkey wallet/keeper.json) --url devnet

# Check balance
solana balance wallet/admin.json  --url devnet
solana balance wallet/keeper.json --url devnet
```

---

## Environment Variables (`.env.example`)

```bash
# Network
RPC_ENDPOINT=https://api.devnet.solana.com
WS_ENDPOINT=wss://api.devnet.solana.com

# Program
PROGRAM_ID=<YOUR_PROGRAM_ID>

# Wallets
ADMIN_KEYPAIR=./wallet/admin.json
KEEPER_KEYPAIR=./wallet/keeper.json

# Keeper bot
MARKET_PUBKEYS=<MARKET_PUBKEY_1>,<MARKET_PUBKEY_2>
POLL_INTERVAL_MS=2000
MIN_FILL_SIZE=1000
```

---

## Open Questions / Deferred Decisions

| # | Topic | Current state | Future path |
|---|-------|--------------|-------------|
| 1 | Oracle for resolution | Admin keypair | Switchboard VRF or Pyth price feeds |
| 2 | Keeper incentives | Flat 5 bps hardcoded | Per-market configurable bps; priority-fee auction |
| 3 | Multi-outcome markets | Binary only | N-outcome needs N internal balance fields or dynamic vec |
| 4 | GTD order expiry | No expiry | Keeper submits `CancelOrder` for expired orders; earns cleanup fee |
| 5 | Price precision | u64 basis points (0–10000) | u64 with 6 decimals if tighter pricing needed |
| 6 | Front-running | Keepers race for fills | JIT window (Drift-style) or commit-reveal scheme |
