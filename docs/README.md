# Prediction Market — Solana Devnet

Build on a fully on-chain, permissionless prediction market. Native Solana program, TypeScript SDK, and keeper-bot infrastructure for binary markets on devnet.

---

## Developer Quickstart

Get your environment set up and place your first order in minutes.

[Get Started →](./getting-started/quickstart.md)

---

## Documentation Structure

### Getting Started

| Page | Description |
|------|-------------|
| [Overview](./getting-started/overview.md) | What this project is and how it differs from Polymarket |
| [How It Works](./getting-started/how-it-works.md) | Architecture, component roles, and data flow |
| [Quickstart](./getting-started/quickstart.md) | Deploy the program, create a market, and place your first order |

### Core Concepts

| Page | Description |
|------|-------------|
| [Markets](./core-concepts/markets.md) | Binary markets, question hashes, lifecycle states |
| [Market Categories](./core-concepts/categories.md) | On-chain `primary_category` / `subcategory` taxonomy (Polymarket-style browse filters) |
| [Prices & Order Book](./core-concepts/prices-and-orderbook.md) | Price representation, DLOB structure, spread, and crossing |
| [Positions & Tokens](./core-concepts/positions-and-tokens.md) | Internal balances, YES/NO tokens, and optional SPL tokenization |
| [Collateral (mock USDC)](./core-concepts/collateral.md) | Devnet USDC, vault model, and rent economics |
| [Order Lifecycle](./core-concepts/order-lifecycle.md) | Place → fill → cancel → redeem end-to-end |
| [Resolution](./core-concepts/resolution.md) | Admin resolution, winning outcome, and payout |
| [Negative Risk Markets](./core-concepts/negative-risk.md) | Complete sets, negative-risk positions, and cross-market collateral conversion |

### SDK

| Page | Description |
|------|-------------|
| [Overview](./sdk/overview.md) | SDK packages, installation, and design philosophy |
| [Quickstart](./sdk/quickstart.md) | Initialize a connection, fetch markets, and submit an order |
| [Orders](./sdk/orders.md) | Build, place, cancel, and query orders |
| [Fees](./sdk/fees.md) | Fill fee flow, keeper incentives, Polymarket / Drift comparison, and roadmap |
| [Outcome Tokens](./sdk/outcome-tokens.md) | Split, merge, and opt-in SPL tokenization |
| [WebSocket](./sdk/websocket.md) | Real-time order book updates via `OrderSubscriber` |

### Keeper Bots

| Page | Description |
|------|-------------|
| [Overview](./keeper/overview.md) | What keeper bots are and why they exist |
| [Getting Started](./keeper/getting-started.md) | Configure and run the keeper daemon |
| [Economics](./keeper/economics.md) | Fill fees, costs, and profitability |
| [Operations](./keeper/operations.md) | Race handling, WebSocket reliability, and monitoring |

### On-Chain Program

| Page | Description |
|------|-------------|
| [Overview](./program/overview.md) | Native Solana program design, p-token CPI savings, and security model |
| [Pinocchio Migration](./program/pinocchio.md) | Upgrade from `solana-program` to Pinocchio for ~95% CU reduction |
| [Instructions](./program/instructions.md) | All 9 instructions with accounts, args, and constraints |
| [Account Structs](./program/accounts.md) | Borsh layout for `Market`, `Event`, `Order`, and `UserPosition` |
| [PDA Seeds](./program/pda-seeds.md) | Seed derivation for all program-owned accounts |

### Resources

| Page | Description |
|------|-------------|
| [Program & Deployment](./resources/program-id.md) | Program ID, deploy commands, and devnet config |
| [Error Codes](./resources/error-codes.md) | All `PredictionMarketError` variants with descriptions |
| [Directory Structure](./resources/directory-structure.md) | Monorepo layout and file responsibilities |
| [Dependencies](./resources/dependencies.md) | Rust crates and TypeScript packages |

---

## Solana Upgrade Highlights (May 2026)

| Upgrade | Status | Impact on this project |
|---------|--------|------------------------|
| **P-Token** ([SIMD-0266](https://github.com/solana-foundation/solana-improvement-documents/pull/266)) | ✅ Devnet active, mainnet targeting May 2026 | 95–98% CU reduction on all vault token transfers; zero code changes required |
| **Pinocchio framework** | ✅ Available | Rewriting the program with Pinocchio further reduces CU + binary size |
| **Token-2022 Extensions** | ✅ Mainnet active | Optional for future YES/NO token composability (transfer hooks, metadata) |

See [Pinocchio Migration Guide](./program/pinocchio.md) to adopt these improvements.

---

## Key Differences from Polymarket

| Dimension | Polymarket | This Project |
|-----------|-----------|--------------|
| Chain | Polygon | Solana Devnet |
| Order Book | Centralized CLOB (off-chain) | On-chain DLOB (decentralized) |
| Order matching | Operator-run | Permissionless keeper bots |
| Censorship risk | Operator can censor | Any keeper can fill any order |
| Contract framework | Solidity + Exchange contract | Native Rust Solana program (no Anchor) |
| Settlement | Polygon Exchange contract | Solana program CPI |
| Collateral | pUSD (Polygon USDC) | Mock USDC (devnet SPL mint) |
| Auth | EIP-712 + HMAC | Solana keypair (Ed25519) |
| Fees | Taker: `feeRate × p×(1−p)` on notional; maker rebate from taker fee; optional maker bps | Same shape + market params on `Market`; splits → maker, keeper, treasury (see [SDK — Fees](./sdk/fees.md)) |

---

*Network: Solana Devnet — not for production use.*
