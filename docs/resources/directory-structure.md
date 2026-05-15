# Directory Structure

> Monorepo layout and file responsibilities.

---

## Top-Level

```
polymarket-forking-solana/
├── PLAN.md                  ← original architecture plan
├── README.md                ← project introduction
├── Cargo.toml               ← Rust workspace manifest
├── Cargo.lock
├── package.json             ← pnpm monorepo root
├── pnpm-workspace.yaml      ← workspace package list
├── .env.example             ← environment variable template
├── .gitignore
│
├── wallet/                  ← devnet keypairs (git-ignored)
│   ├── admin.json
│   └── keeper.json
│
├── program/                 ← native Solana program (Rust)
├── sdk/                     ← TypeScript SDK
├── keeper/                  ← keeper bot daemon
├── app/                     ← Next.js frontend
├── scripts/                 ← devnet utility scripts
├── tests/                   ← program + DLOB tests
└── docs/                    ← this documentation
```

---

## `program/` — Native Solana Program

```
program/
├── Cargo.toml               ← crate-type = ["cdylib", "lib"]
└── src/
    ├── entrypoint.rs        ← entrypoint!(process_instruction)
    ├── processor.rs         ← discriminant → instruction handler dispatch
    ├── instruction.rs       ← InstructionData enum + arg structs (Borsh)
    ├── error.rs             ← PredictionMarketError enum
    ├── lib.rs               ← pub mod declarations
    ├── state/
    │   ├── mod.rs           ← pub use + discriminant constants
    │   ├── market.rs        ← Market struct (295 bytes)
    │   ├── order.rs         ← Order struct (107 bytes)
    │   ├── user_position.rs ← UserPosition struct (1131 bytes)
    │   └── event.rs         ← Event struct (592 bytes)
    ├── instructions/
    │   ├── mod.rs
    │   ├── create_market.rs
    │   ├── split.rs
    │   ├── merge.rs
    │   ├── place_order.rs
    │   ├── cancel_order.rs
    │   ├── fill_order.rs
    │   ├── resolve_market.rs
    │   ├── redeem.rs
    │   ├── tokenize_position.rs
    │   ├── create_event.rs       ← instruction 9
    │   ├── add_market_to_event.rs ← instruction 10
    │   └── resolve_event.rs      ← instruction 11
    └── utils/
        ├── mod.rs
        ├── pda.rs           ← seed constants + find_program_address wrappers
        └── token.rs         ← SPL token CPI helpers
```

---

## `sdk/` — TypeScript SDK

```
sdk/
├── package.json             ← name: @polymarket-sol/sdk
├── tsconfig.json
└── src/
    ├── index.ts             ← re-exports everything public
    ├── constants.ts         ← PROGRAM_ID, fee defaults / legacy bps, IX discriminant map
    ├── types.ts             ← Market, Order, UserPosition, Event interfaces + enums
    ├── instructions.ts      ← TransactionInstruction builders for all 12 instructions
    ├── accounts.ts          ← Borsh deserializers + RPC fetchers (incl. fetchEvent)
    ├── pda.ts               ← findMarketPda, findOrderPda, findUserPositionPda, findEventPda
    ├── dlob/
    │   ├── DLOB.ts          ← In-memory sorted order book (bids + asks)
    │   ├── DLOBNode.ts      ← Wraps Order with remaining / applyFill helpers
    │   └── OrderSubscriber.ts ← getProgramAccounts + WebSocket subscription
    └── utils/
        └── math.ts          ← Price conversion (bps ↔ decimal)
```

---

## `keeper/` — Keeper Bot

```
keeper/
├── package.json
├── tsconfig.json
└── src/
    ├── index.ts             ← main daemon loop; runMarket() per market
    ├── Filler.ts            ← simulate → send FillOrder txns
    └── config.ts            ← reads env vars → KeeperConfig
```

---

## `app/` — Next.js Frontend

```
app/
├── package.json
├── next.config.js
├── tailwind.config.ts
├── postcss.config.js
├── tsconfig.json
└── src/
    ├── pages/
    │   ├── _app.tsx         ← WalletProvider setup
    │   ├── index.tsx        ← market list page
    │   └── market/
    │       └── [id].tsx     ← market detail (order book + trade panel)
    ├── components/
    │   ├── OrderBook.tsx    ← bid/ask ladder display
    │   ├── PlaceOrder.tsx   ← order form (side, price, size)
    │   └── PositionPanel.tsx ← user position display
    ├── hooks/
    │   ├── useMarkets.ts    ← fetch all Market accounts
    │   └── useOrderBook.ts  ← wraps OrderSubscriber for React
    └── lib/
        └── wallet.tsx       ← @solana/wallet-adapter config
```

---

## `scripts/` — Devnet Utilities

```
scripts/
├── package.json
├── tsconfig.json
├── _common.ts               ← shared connection + wallet loading
├── create-market.ts         ← CreateMarket instruction script
├── fund-wallet.ts           ← mock USDC mint + airdrop script
├── resolve-market.ts        ← ResolveMarket instruction script
├── create-event.ts          ← CreateEvent + AddMarketToEvent script
└── resolve-event.ts         ← ResolveEvent instruction script
```

---

## `tests/`

```
tests/
├── program/
│   └── prediction_market.rs ← Rust integration tests (solana-program-test)
└── keeper/
    ├── jest.config.js
    └── dlob.test.ts         ← Jest unit tests for DLOB in-memory logic
```

---

## `docs/` — This Documentation

```
docs/
├── README.md                ← landing page + navigation
├── getting-started/
│   ├── overview.md
│   ├── how-it-works.md
│   └── quickstart.md
├── core-concepts/
│   ├── markets.md
│   ├── events.md
│   ├── prices-and-orderbook.md
│   ├── positions-and-tokens.md
│   ├── collateral.md
│   ├── order-lifecycle.md
│   └── resolution.md
├── sdk/
│   ├── overview.md
│   ├── quickstart.md
│   ├── orders.md
│   ├── fees.md
│   ├── outcome-tokens.md
│   └── websocket.md
├── keeper/
│   ├── overview.md
│   ├── getting-started.md
│   ├── economics.md
│   └── operations.md
├── program/
│   ├── overview.md
│   ├── instructions.md
│   ├── accounts.md
│   └── pda-seeds.md
└── resources/
    ├── program-id.md
    ├── error-codes.md
    ├── directory-structure.md
    └── dependencies.md
```
