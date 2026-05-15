# Collateral (mock USDC)

> Devnet USDC, vault model, and rent economics for on-chain accounts.

---

## Collateral Token

This project uses a **mock USDC SPL token** deployed on Solana Devnet. It has 6 decimal places, matching mainnet USDC (1 USDC = 1,000,000 units).

The mint address is set at program deploy time and stored in `sdk/src/constants.ts`. It is recorded on each `Market` account as `collateral_mint`.

> **Devnet only.** This is not real money. Use `solana airdrop` and `scripts/fund-wallet.ts` to get test tokens.

---

## Vault Model

Each market has exactly **one collateral vault** — a USDC Associated Token Account (ATA) owned by the `vault_authority` PDA.

```
vault_authority PDA seeds: [b"vault_authority", market_pubkey]
```

All user deposits (`Split`) transfer USDC into this vault. All user withdrawals (`Merge`, `Redeem`) transfer USDC out of this vault. The program uses `invoke_signed` with the vault_authority PDA seeds to authorize outbound transfers.

**Why one vault per market?**

- Simplifies accounting — no per-user escrow accounts needed.
- Reduces ATA rent to a single fixed cost per market.
- The vault balance always equals the total outstanding YES + NO internal balances across all users.

---

## Depositing Collateral — Split

`Split(amount)` deposits USDC into the vault and credits both YES and NO balances equally:

```
Split(100 USDC)
  USDC [user wallet]  →  vault
  UserPosition.yes_balance += 100 USDC
  UserPosition.no_balance  += 100 USDC
```

The total value of (YES + NO) always equals the original deposit. You can always get your collateral back by merging.

---

## Withdrawing Collateral — Merge

`Merge(amount)` is the reverse of Split: burn equal YES and NO balances, return collateral:

```
Merge(50 USDC)
  UserPosition.yes_balance -= 50 USDC
  UserPosition.no_balance  -= 50 USDC
  vault  →  USDC [user wallet]
```

Merge is allowed any time before and after resolution (as long as the user holds both sides).

---

## Withdrawing Collateral — Redeem (Post-Resolution)

After a market resolves, holders of the **winning side** redeem at 1:1:

```
Market resolves YES
  Redeem(100 YES)
    UserPosition.yes_balance -= 100 USDC
    vault  →  100 USDC [user wallet]

Losing side (NO) has no redemption value.
```

---

## Rent Economics

Solana accounts must hold enough SOL to be rent-exempt. Accounts in this program:

| Account | Size | Approx. rent-exempt SOL |
|---------|------|------------------------|
| Market | 212 bytes | ~0.0015 SOL |
| Order | 107 bytes | ~0.0010 SOL |
| UserPosition | 1131 bytes | ~0.0085 SOL |
| USDC vault ATA | 165 bytes | ~0.0020 SOL |

### Who pays what

| Account | Payer | Returned when |
|---------|-------|---------------|
| Market PDA | Admin (at `CreateMarket`) | Never (market is permanent) |
| USDC vault ATA | Admin (at `CreateMarket`) | Never |
| UserPosition PDA | User (at first `Split`) | Never (persists across trades) |
| Order PDA | User (at `PlaceOrder`) | When order is fully filled or cancelled |

Order rent is recovered on fill: when the keeper closes a fully-filled order account, the lamports are transferred to the **keeper's UserPosition** account as an additional incentive. On cancel, rent returns to the order owner.

---

## P-Token: CU Savings on Vault Operations

Every `Split`, `Merge`, and `Redeem` instruction makes one CPI call to the SPL Token `transfer` instruction to move USDC between the user's ATA and the market vault.

**P-Token** ([live on devnet since April 2026](https://solana.com/upgrades/p-token)) is a drop-in replacement for the SPL Token program that reduces the CU cost of these calls by **~98%**:

| Token CPI | Before p-token | After p-token | Instruction |
|-----------|----------------|---------------|-------------|
| USDC deposit (Split) | 4,645 CU | 76 CU | `Transfer` |
| USDC withdrawal (Merge / Redeem) | 4,645 CU | 76 CU | `Transfer` |

No code changes are required — p-token uses the same program address as SPL Token, so all existing CPI calls automatically use the optimized implementation after the feature gate activates.

This means every user interaction involving real USDC movement is now dramatically cheaper on devnet.

---

## Next Steps

- [Order Lifecycle](./order-lifecycle.md)
- [Positions & Tokens](./positions-and-tokens.md)
- [Program — P-Token & CPI Efficiency](../program/overview.md#p-token--cpi-efficiency)
