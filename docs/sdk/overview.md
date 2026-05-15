# SDK Overview

> TypeScript client for the prediction-market Solana program. No IDL, no Anchor — hand-written types that mirror the on-chain Borsh layout exactly.

---

## Package

The SDK is a workspace package located at `sdk/` and published internally as `@polymarket-sol/sdk`.

```bash
# From project root (pnpm workspace)
pnpm install

# Import in your TypeScript code
import { splitInstruction, findMarketPda } from "@polymarket-sol/sdk";
```

---

## Design Philosophy

Polymarket's SDK wraps a REST API with HMAC authentication and EIP-712 order signing. This SDK wraps **direct Solana RPC calls** and **hand-crafted `TransactionInstruction` builders**.

| Dimension | Polymarket SDK | This SDK |
|-----------|---------------|----------|
| Transport | HTTPS REST + WebSocket | Solana RPC + WebSocket |
| Auth | HMAC-SHA256 (L2) + EIP-712 (orders) | Ed25519 keypair (Solana wallet) |
| Type generation | Auto from OpenAPI spec | Hand-written, mirrors Borsh layout |
| Order serialization | JSON payload | Borsh binary (`Buffer`) |
| Matching | Server-side | Client-side DLOB + keeper bot |

**No IDL, no auto-generation.** Because there is no Anchor, there is no IDL file. All TypeScript types in `types.ts` are written by hand to exactly match the Rust struct field order and sizes. All instruction builders in `instructions.ts` manually encode bytes with the correct discriminant and little-endian integers.

---

## Module Structure

| Module | Description |
|--------|-------------|
| `instructions.ts` | `TransactionInstruction` builders for all 9 program instructions |
| `accounts.ts` | Borsh deserializers + RPC fetchers for Market, Order, UserPosition |
| `pda.ts` | `findMarketPda`, `findOrderPda`, `findUserPositionPda`, `findVaultAuthorityPda` |
| `types.ts` | TypeScript interfaces and enums mirroring Rust structs |
| `constants.ts` | `PROGRAM_ID`, `FILL_FEE_BPS`, instruction discriminant map |
| `dlob/DLOB.ts` | In-memory order book (sorted bids + asks) |
| `dlob/DLOBNode.ts` | Wrapper around an `Order` with `remaining`, `applyFill` helpers |
| `dlob/OrderSubscriber.ts` | `getProgramAccounts` + WebSocket subscription for real-time order feeds |
| `utils/math.ts` | Price conversion helpers (bps ↔ decimal) |

---

## Installation

The SDK is not published to npm. Install it as a pnpm workspace dependency:

```jsonc
// your package.json
{
  "dependencies": {
    "@polymarket-sol/sdk": "workspace:*"
  }
}
```

Required peer dependencies:

```bash
pnpm add @solana/web3.js @solana/spl-token
```

---

## Peer Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `@solana/web3.js` | `^1` | `Connection`, `PublicKey`, `Transaction`, `TransactionInstruction` |
| `@solana/spl-token` | `^0.3` | ATA address derivation, token constants |

---

## Next Steps

- [SDK Quickstart](./quickstart.md) — connect and place your first order
- [SDK Orders](./orders.md) — all instruction builders
- [SDK WebSocket](./websocket.md) — real-time order book
