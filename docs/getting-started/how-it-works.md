# How It Works

> Architecture, component roles, and the data flow from order placement to settlement.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Solana Devnet                            │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │         prediction-market Program (native, no Anchor)     │  │
│  │                                                           │  │
│  │  Market Account   Order Account    UserPosition Account   │  │
│  │  ─────────────    ─────────────    ────────────────────   │  │
│  │  question_hash    side (bid/ask)   yes_balance            │  │
│  │  vault            price (u64)      no_balance             │  │
│  │  collateral_mint  size (u64)       locked_yes             │  │
│  │  resolved         fill_amount      locked_no              │  │
│  │  winning_outcome  user_pubkey      locked_collateral      │  │
│  │  admin            market_pubkey    open_orders[]          │  │
│  │  end_time         nonce                                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Only 1 ATA per market: USDC vault (admin pays at creation)     │
└─────────────────────────────────────────────────────────────────┘
         ▲                          ▲
         │ RPC / WebSocket          │ submit fill txns
         │                          │
┌────────┴────────┐        ┌────────┴────────────────────────────┐
│   Next.js App   │        │          DLOB Keeper Bot            │
│   (frontend)    │        │                                     │
│                 │        │  OrderSubscriber (WebSocket/poll)   │
│  wallet adapter │        │       │                             │
│  place order    │        │       ▼                             │
│  view markets   │        │  In-Memory DLOB                     │
│  view positions │        │  ├─ bids: sorted DESC by price      │
│  resolve market │        │  └─ asks: sorted ASC by price       │
│                 │        │       │                             │
└─────────────────┘        │       ▼ (cross-spread detection)    │
                           │  Filler: submit fill instruction    │
                           └─────────────────────────────────────┘
```

---

## Component Roles

### 1. Native Solana Program (`program/`)

The on-chain source of truth. All state transitions — creating markets, placing orders, filling orders, resolving markets, redeeming winnings — require a signed Solana transaction processed by this program.

**Program structure:**

| File | Responsibility |
|------|----------------|
| `entrypoint.rs` | `entrypoint!(process_instruction)` macro — Solana program entry point |
| `processor.rs` | Matches 1-byte discriminant → dispatches to instruction handler |
| `instruction.rs` | `InstructionData` enum with Borsh-serialized args per variant |
| `state/` | Borsh-serializable account structs with fixed-size layouts |
| `error.rs` | `PredictionMarketError` enum with error codes |
| `utils/pda.rs` | PDA seed constants and `find_program_address` wrappers |

Every instruction handler validates accounts manually: ownership checks, signer checks, PDA derivation checks, and business-logic constraints — no Anchor macros.

**State accounts:**

| Account | One per | Holds |
|---------|---------|-------|
| `Market` | question | vault, resolution state, admin, end time |
| `Order` | open order | side, price, size, fill amount, nonce |
| `UserPosition` | (user, market) pair | YES/NO balances, locked amounts, open order list |

---

### 2. DLOB Keeper Bot (`keeper/`)

A TypeScript daemon that provides decentralized order matching. Any wallet can run a keeper — there is no whitelist or admin gate on `FillOrder`.

**Flow:**

1. **`OrderSubscriber`** — calls `getProgramAccounts` with two `memcmp` filters (order discriminant + market pubkey) to fetch all resting orders for a market. Subscribes to account changes via WebSocket.
2. **`DLOB`** — maintains two sorted lists per market in memory: bids sorted DESC by price (best bid first), asks sorted ASC by price (best ask first). FIFO within the same price level.
3. **Crossing detection** — on every DLOB mutation, checks if `best_bid.price >= best_ask.price`. When true, a fill is possible.
4. **`Filler`** — simulates the `FillOrder` transaction first (skips if simulation fails). Submits via `sendAndConfirmTransaction`. First keeper to land the transaction earns the fill fee.

---

### 3. TypeScript SDK (`sdk/`)

A hand-written TypeScript client with no IDL or auto-generation. All types mirror the Borsh layout of the Rust structs byte-for-byte.

| Module | Contents |
|--------|----------|
| `instructions.ts` | Builders for all 9 `TransactionInstruction` variants |
| `accounts.ts` | Borsh deserializers for `Market`, `Order`, `UserPosition` |
| `pda.ts` | `findMarketPda`, `findOrderPda`, `findUserPositionPda` |
| `types.ts` | TypeScript interfaces mirroring Rust structs |
| `dlob/` | `DLOB`, `DLOBNode`, `OrderSubscriber` |
| `utils/math.ts` | Price conversion helpers |

---

### 4. Frontend (`app/`)

A Next.js app using `@solana/wallet-adapter` and the SDK directly. Supports Phantom, Backpack, and Solflare.

Pages: market list, market detail (order book ladder, trade panel, position panel).

Real-time order book updates are powered by `useOrderBook` — a React hook that wraps `OrderSubscriber`.

---

## Full Data Flow: Place to Settle

```
1. User signs PlaceOrder tx (frontend / SDK)
   └─ Program creates Order PDA, locks collateral in UserPosition

2. Keeper bot detects new Order account (WebSocket)
   └─ Updates in-memory DLOB

3. Cross detected (best_bid.price >= best_ask.price)
   └─ Keeper simulates FillOrder tx

4. Keeper submits FillOrder tx
   └─ Program validates crossing and maker/taker ordering
   └─ Applies taker curve fee, optional maker fee, maker rebate, keeper + treasury split
   └─ Closes fully-filled Order accounts (rent → user)

5. Admin calls ResolveMarket (outcome: YES or NO)
   └─ Program sets market.resolved = true, market.winning_outcome

6. Winner calls Redeem
   └─ Program burns winning YES/NO balance
   └─ Transfers USDC from vault to user's ATA
```
