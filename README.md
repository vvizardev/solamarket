# Polymarket Fork on Solana — DLOB Edition

A Polymarket-style binary prediction market on **Solana devnet**, replacing the centralized CLOB with a **Decentralised Limit Order Book (DLOB)** modelled after Drift Protocol v2.

Built as a **native Solana program** (no Anchor) with a hand-written TypeScript SDK, a permissionless keeper bot, and a Next.js frontend.

---

## Architecture

```
┌─────────────────────────────────┐
│        Solana Devnet             │
│  program/  (native, no Anchor)  │
│  Market · Order · UserPosition  │
└──────────────┬──────────────────┘
               │ RPC / WebSocket
       ┌───────┴────────┐
       │                │
  app/ (Next.js)   keeper/ (TypeScript daemon)
  wallet adapter   OrderSubscriber → DLOB → Filler
```

| Component | Path | Description |
|---|---|---|
| On-chain program | `program/` | 9 instructions, Borsh state, manual account validation |
| TypeScript SDK | `sdk/` | Hand-written deserializers + instruction builders |
| Keeper bot | `keeper/` | Watches orders, fills crossing bids/asks |
| Frontend | `app/` | Next.js order book UI |
| Scripts | `scripts/` | CLI tools for devnet operations |

---

## Prerequisites

| Tool | Install |
|---|---|
| Rust + Cargo | `curl https://sh.rustup.rs -sSf \| sh` |
| Solana CLI | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` |
| Node.js ≥ 18 | https://nodejs.org |
| pnpm | `npm i -g pnpm` |

---

## Quick Start

### 1. Clone & install

```bash
git clone <repo-url>
cd polymarket-forking-solana
pnpm install
```

### 2. Configure Solana CLI for devnet

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana config set --url devnet
```

### 3. Create wallets

```bash
mkdir -p wallet
solana-keygen new -o wallet/admin.json  --no-bip39-passphrase
solana-keygen new -o wallet/keeper.json --no-bip39-passphrase
```

### 4. Airdrop SOL

```bash
solana airdrop 2 $(solana-keygen pubkey wallet/admin.json)  --url devnet
solana airdrop 1 $(solana-keygen pubkey wallet/keeper.json) --url devnet
```

### 5. Build the program

```bash
cargo build-sbf --manifest-path program/Cargo.toml
```

### 6. Deploy to devnet

```bash
solana program deploy \
  target/deploy/prediction_market.so \
  --keypair wallet/admin.json \
  --url devnet
```

Copy the printed **Program ID** and paste it into `sdk/src/constants.ts`:

```ts
// sdk/src/constants.ts
export const PROGRAM_ID = new PublicKey("<YOUR_PROGRAM_ID>");
```

### 7. Create a mock USDC mint & fund wallets

```bash
cp .env.example .env
# Edit .env and set PROGRAM_ID

PROGRAM_ID=<id> pnpm --filter scripts fund-wallet
# → prints COLLATERAL_MINT=<mint_address>
# Add COLLATERAL_MINT to .env
```

### 8. Create a market

```bash
PROGRAM_ID=<id> \
COLLATERAL_MINT=<mint> \
QUESTION="Will BTC exceed \$100k by end of 2025?" \
END_TIME=1777000000 \
pnpm --filter scripts create-market
# → prints market PDA address
```

### 9. Start the keeper

```bash
# Add market address to .env: MARKET_PUBKEYS=<market_pda>
pnpm --filter keeper dev
```

### 10. Start the frontend

```bash
pnpm --filter app dev
# Open http://localhost:3000
```

---

## Environment Variables

Copy `.env.example` to `.env` and fill in each value:

```env
RPC_ENDPOINT=https://api.devnet.solana.com   # replace with Helius/Alchemy for reliability
WS_ENDPOINT=wss://api.devnet.solana.com
PROGRAM_ID=                                   # set after deploy
COLLATERAL_MINT=                              # set after fund-wallet
KEEPER_KEYPAIR=../../wallet/keeper.json
MARKET_PUBKEYS=                               # comma-separated market PDAs
NEXT_PUBLIC_RPC_ENDPOINT=https://api.devnet.solana.com
```

---

## Running Tests

### Rust unit tests (no validator required)

```bash
cargo test --manifest-path program/Cargo.toml
```

Tests cover:
- Instruction discriminant byte values
- Account struct byte-layout sizes
- PDA derivation determinism
- `Order.remaining()` and fill logic
- `UserPosition` open-order list (add / swap-and-pop remove)

### TypeScript DLOB unit tests

```bash
cd tests/keeper
pnpm install
pnpm jest
```

Tests cover:
- Bid/ask insertion and removal
- Best bid (price DESC) / best ask (price ASC)
- Cross detection (`best_bid.price >= best_ask.price`)
- FIFO ordering within the same price level
- `DLOBNode` fill caching and reconcile

---

## Project Structure

```
polymarket-forking-solana/
├── program/src/
│   ├── entrypoint.rs          entrypoint!(process_instruction)
│   ├── processor.rs           dispatch by InstructionData variant
│   ├── instruction.rs         9-variant enum, Borsh-serialised
│   ├── error.rs               PredictionMarketError (thiserror)
│   ├── state/
│   │   ├── market.rs          Market account  (212 bytes)
│   │   ├── order.rs           Order account   (107 bytes)
│   │   └── user_position.rs   UserPosition    (1 131 bytes)
│   ├── instructions/          one file per instruction handler
│   └── utils/
│       ├── pda.rs             seed constants + find_*_pda helpers
│       └── token.rs           hand-encoded SPL Token CPI (no spl-token crate)
│
├── sdk/src/
│   ├── accounts.ts            byte-exact Borsh deserializers
│   ├── instructions.ts        TransactionInstruction builders
│   ├── pda.ts                 PDA derivation (mirrors on-chain seeds)
│   ├── types.ts               TypeScript types mirroring Rust structs
│   ├── constants.ts           PROGRAM_ID, discriminants
│   └── dlob/
│       ├── DLOB.ts            in-memory sorted order book
│       ├── DLOBNode.ts        per-order node with fill cache
│       └── OrderSubscriber.ts getProgramAccounts snapshot + WebSocket
│
├── keeper/src/
│   ├── index.ts               daemon loop, one subscriber per market
│   ├── Filler.ts              simulate → send → handle race conditions
│   └── config.ts              env-var config loader
│
├── app/src/
│   ├── pages/index.tsx        market list
│   ├── pages/market/[id].tsx  order book + trade panel + positions
│   ├── components/
│   │   ├── OrderBook.tsx      live bid/ask ladder with depth bars
│   │   ├── PlaceOrder.tsx     side toggle, price/size form
│   │   └── PositionPanel.tsx  balances, locked amounts, redeem button
│   └── hooks/
│       ├── useMarkets.ts      getProgramAccounts for all Market PDAs
│       └── useOrderBook.ts    wraps OrderSubscriber, syncs React state
│
├── scripts/
│   ├── fund-wallet.ts         create mock USDC mint, airdrop tokens
│   ├── create-market.ts       CreateMarket instruction
│   └── resolve-market.ts      ResolveMarket instruction
│
└── tests/
    ├── program/               Rust integration tests (solana-program-test)
    └── keeper/                Jest DLOB unit tests
```

---

## Instructions Reference

| # | Instruction | Caller | Effect |
|---|---|---|---|
| 0 | `CreateMarket` | admin | Create Market PDA + USDC vault ATA |
| 1 | `Split` | user | Deposit USDC → credit YES + NO balances |
| 2 | `Merge` | user | Burn YES + NO → withdraw USDC |
| 3 | `PlaceOrder` | user | Create Order PDA, lock balance |
| 4 | `CancelOrder` | user | Close Order PDA, release locked balance |
| 5 | `FillOrder` | keeper | Match crossing orders, swap balances, pay 5 bps fee |
| 6 | `ResolveMarket` | admin | Set winning outcome (1=YES, 2=NO) |
| 7 | `Redeem` | user | Burn winning balance → receive USDC |
| 8 | `TokenizePosition` | user | Opt-in: mint real SPL YES/NO tokens |

---

## Key Design Decisions

**No Anchor** — manual account validation keeps the binary small and compute budget low. Every handler explicitly checks ownership, PDA derivation, and signers.

**Internal balances** — YES/NO balances live inside `UserPosition` (a program PDA), not in SPL ATAs. This avoids ~0.002 SOL rent per user per market. SPL tokens are only minted on opt-in via `TokenizePosition`.

**No spl-token crate dependency** — SPL Token CPI calls are hand-encoded in `utils/token.rs` to avoid conflicting transitive `solana-program` versions in the workspace.

**Permissionless filling** — any keypair can run a keeper. First to land the `FillOrder` transaction earns 5 bps of the fill size. The program validates the cross independently; a keeper cannot steal funds.
