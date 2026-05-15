# Instructions Reference

> All 9 program instructions with account lists, argument schemas, and security constraints.

---

## Encoding Format

Every instruction is Borsh-encoded:

```
[discriminant: u8] [args: Borsh-serialized struct]
```

The TypeScript SDK handles encoding automatically via `instructions.ts`. If calling the program directly:

```typescript
// First byte = discriminant, rest = Borsh-serialized args
const data = Buffer.alloc(1 + argsSize);
data.writeUInt8(discriminant, 0);
borsh.serialize(ArgsSchema, args, data.slice(1));
```

---

## 0 — CreateMarket

**Caller:** Admin only.

Creates the Market PDA and initializes the USDC vault ATA.

**Arguments:**

| Field | Type | Description |
|-------|------|-------------|
| `question_hash` | `[u8; 32]` | SHA-256 hash of the question string |
| `end_time` | `i64` | Unix timestamp; no new orders after this |
| `fee_recipient_user` | `Pubkey` | Treasury owner for `taker_fee` remainder + `maker_fee` (non-default in production) |
| `taker_curve_numer` / `taker_curve_denom` | `u32` / `u32` | Polymarket-style curve scalars (see [SDK — Fees](../sdk/fees.md); `0` numer disables curve in favor of legacy keeper path) |
| `maker_fee_bps` | `u16` | Optional maker fee on `fill_cost` |
| `maker_rebate_of_taker_bps` | `u16` | Share of `taker_fee` paid to maker |
| `keeper_reward_of_taker_bps` | `u16` | Share of `taker_fee` paid to keeper |

Exact Borsh packing can mirror the tail fields of [Market](./accounts.md#market-292-bytes); if `CreateMarket` stays minimal in an early build, the program can initialize these from sane defaults and add an `UpdateMarketFees` instruction later.

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | admin |
| 1 | writable | market PDA |
| 2 | writable | vault ATA |
| 3 | — | vault_authority PDA |
| 4 | — | collateral_mint |
| 5 | — | system_program |
| 6 | — | token_program |
| 7 | — | associated_token_program |
| 8 | — | rent sysvar |

**Constraints:** Admin signer; `fee_recipient_user != Pubkey::default()` in production; `maker_rebate_of_taker_bps + keeper_reward_of_taker_bps <= 10_000`; admin pubkeys stored as today.

---

## 1 — Split

**Caller:** Any user.

Deposits USDC into the vault; credits `yes_balance` and `no_balance` equally.

**Arguments:** `amount: u64` (USDC units)

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | user |
| 1 | writable | market PDA |
| 2 | writable | user_position PDA (created here if first deposit) |
| 3 | writable | user USDC ATA |
| 4 | writable | market vault ATA |
| 5 | — | vault_authority PDA |
| 6 | — | token_program |
| 7 | — | system_program |

**Constraints:** amount > 0; market not resolved; market not expired.

---

## 2 — Merge

**Caller:** Any user.

Burns equal YES + NO balance; returns USDC from vault.

**Arguments:** `amount: u64`

**Accounts:** Same layout as Split.

**Constraints:** amount > 0; `yes_balance >= amount`; `no_balance >= amount`.

---

## 3 — PlaceOrder

**Caller:** Any user.

Creates an Order PDA and locks the appropriate balance.

**Arguments:**

| Field | Type | Description |
|-------|------|-------------|
| `side` | `u8` | `0` = bid (buy YES), `1` = ask (sell YES) |
| `price` | `u64` | Basis points, 1–9999 |
| `size` | `u64` | Collateral units |
| `nonce` | `u64` | Client-chosen unique ID (determines Order PDA) |

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | user |
| 1 | — | market PDA |
| 2 | writable | user_position PDA |
| 3 | writable | order PDA (created here) |
| 4 | — | system_program |

**Constraints:**
- Market not resolved; `end_time` not past.
- Price in range 1–9999.
- Size > 0.
- Sufficient free balance (bid: `locked_collateral + size <= deposited`; ask: `locked_yes + size <= yes_balance`).
- `open_order_count < 32`.

---

## 4 — CancelOrder

**Caller:** Order owner only.

Closes the Order PDA; releases locked balance.

**Arguments:** `nonce: u64`

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | user (must be order.user) |
| 1 | — | market PDA |
| 2 | writable | user_position PDA |
| 3 | writable | order PDA |

**Constraints:** `user_ai.key == order.user`; order PDA must exist.

---

## 5 — FillOrder

**Caller:** Any keeper (permissionless).

Validates two crossing orders and swaps balances, applying **Polymarket-style** economics: **taker fee** (curve), optional **maker fee**, **maker rebate**, **keeper reward**, and **treasury** accrual to the market’s `fee_recipient` `UserPosition`. See [SDK — Fees](../sdk/fees.md).

**Arguments:** `fill_size: u64`

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | keeper |
| 1 | — | market PDA |
| 2 | writable | bid_order PDA |
| 3 | writable | ask_order PDA |
| 4 | writable | bid UserPosition |
| 5 | writable | ask UserPosition |
| 6 | writable | keeper UserPosition (receives `keeper_reward` share of taker fee) |
| 7 | writable | `fee_recipient` UserPosition PDA for `(market, market.fee_recipient_user)` (credits `treasury_share + maker_fee`) |

**Constraints:**
- `keeper.is_signer`.
- `bid.market == ask.market == market_ai.key`.
- `bid.side == 0`, `ask.side == 1`.
- `bid.price >= ask.price`.
- `fill_size <= bid.remaining` and `fill_size <= ask.remaining`.
- **Maker/taker:** older `Order.created_at` is maker; tie-break by lexicographic order of order account pubkeys.
- `maker_rebate_of_taker_bps + keeper_reward_of_taker_bps <= 10_000`.
- Account #7 must be the `UserPosition` for `(market, market.fee_recipient_user)`. If `fee_recipient_user` equals the **keeper owner**, indices **#6** and **#7** may be the **same** PDA.
- `CreateMarket` MUST initialize `fee_recipient_user` to a real treasury owner (reject `Pubkey::default()` for production deployments).
- All PDA derivations match expected seeds.

---

## 6 — ResolveMarket

**Caller:** Admin only.

Sets market as resolved with winning outcome.

**Arguments:** `outcome: u8` (1 = YES, 2 = NO)

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | signer | admin (must equal market.admin) |
| 1 | writable | market PDA |

**Constraints:** `user_ai.key == market.admin`; `!market.resolved`; outcome ∈ {1, 2}.

---

## 7 — Redeem

**Caller:** Any user (post-resolution).

Exchanges winning outcome balance for USDC at 1:1.

**Arguments:** `amount: u64`

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | user |
| 1 | — | market PDA |
| 2 | writable | user_position PDA |
| 3 | writable | user USDC ATA |
| 4 | writable | market vault ATA |
| 5 | — | vault_authority PDA |
| 6 | — | token_program |

**Constraints:** market.resolved == true; amount > 0; sufficient winning balance.

---

## 8 — TokenizePosition

**Caller:** Any user (opt-in).

Converts internal YES/NO balances to real SPL tokens.

**Arguments:** `amount: u64`

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | user |
| 1 | writable | market PDA |
| 2 | writable | user_position PDA |
| 3 | writable | yes_mint |
| 4 | writable | no_mint |
| 5 | writable | user YES ATA |
| 6 | writable | user NO ATA |
| 7 | — | yes_mint_authority PDA |
| 8 | — | no_mint_authority PDA |
| 9 | — | system_program |
| 10 | — | token_program |
| 11 | — | associated_token_program |
| 12 | — | rent sysvar |

**Constraints:** amount > 0; `yes_balance >= amount`; `no_balance >= amount`.

---

## 9 — CreateEvent

**Caller:** Admin only.

Creates an Event PDA that groups multiple markets under a shared label, end time, and exclusivity mode.

**Arguments:**

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `[u8; 32]` | SHA-256 hash of the event label string |
| `end_time` | `i64` | Unix timestamp; should match the `end_time` of all child markets |
| `is_exclusive` | `bool` | If `true`, `ResolveEvent` will force all non-winning markets to NO |

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | writable, signer | admin |
| 1 | writable | event PDA `[b"event", event_id]` |
| 2 | — | system_program |

**Constraints:** Signer becomes `event.admin`. The event PDA must not already exist.

---

## 10 — AddMarketToEvent

**Caller:** Admin only (must be both `event.admin` and `market.admin`).

Links an existing market to an event. Sets `market.event = event_pubkey` and appends the market pubkey into `event.markets[market_count]`, then increments `market_count`.

**Arguments:** None — accounts identify the market and event.

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | signer | admin |
| 1 | writable | event PDA |
| 2 | writable | market PDA |

**Constraints:**
- `admin.key == event.admin` (returns `NotEventAdmin` otherwise).
- `market.admin == event.admin` (returns `EventAdminMismatch` otherwise).
- `market.event == Pubkey::default()` — market must not already be in an event (returns `MarketAlreadyInEvent`).
- `event.market_count < 16` (returns `EventFull`).
- `!event.resolved` (returns `EventAlreadyResolved`).

---

## 11 — ResolveEvent

**Caller:** Admin only (`event.admin`).

Resolves an exclusive multi-market event atomically in a single transaction. Sets the market at `winning_index` to YES, and all other markets in the event to NO. Also sets `event.resolved = true`.

For **non-exclusive** events (`is_exclusive = false`), this instruction is not needed — resolve each market individually with `ResolveMarket` (instruction 6).

**Arguments:**

| Field | Type | Description |
|-------|------|-------------|
| `winning_index` | `u8` | Index into `event.markets[]` for the YES outcome |

**Accounts:**

| # | Access | Account |
|---|--------|---------|
| 0 | signer | admin |
| 1 | writable | event PDA |
| 2..N | writable | all `event.market_count` market PDAs, in the same order as `event.markets[]` |

The number of market accounts passed must equal `event.market_count`. The program iterates all provided market accounts, resolves `markets[winning_index]` as YES, and the rest as NO.

**Constraints:**
- `admin.key == event.admin` (returns `NotEventAdmin`).
- `!event.resolved` (returns `EventAlreadyResolved`).
- `event.is_exclusive == true` (non-exclusive events must use per-market `ResolveMarket`).
- `winning_index < event.market_count` (returns `InvalidMarketIndex`).
- Each provided market PDA must match `event.markets[i]` (returns `EventMarketMismatch`).
- No market in the event may already be resolved (returns `MarketAlreadyResolved`).

---

## Next Steps

- [Account Structs](./accounts.md)
- [PDA Seeds](./pda-seeds.md)
- [Error Codes](../resources/error-codes.md)
- [Events](../core-concepts/events.md)
