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

**Constraints:** None beyond signer check. Admin is set to the signing key permanently.

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

Validates two crossing orders and swaps balances.

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
| 6 | writable | keeper UserPosition (receives fill fee + rent) |

**Constraints:**
- `keeper.is_signer`.
- `bid.market == ask.market == market_ai.key`.
- `bid.side == 0`, `ask.side == 1`.
- `bid.price >= ask.price`.
- `fill_size <= bid.remaining` and `fill_size <= ask.remaining`.
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

## Next Steps

- [Account Structs](./accounts.md)
- [PDA Seeds](./pda-seeds.md)
- [Error Codes](../resources/error-codes.md)
