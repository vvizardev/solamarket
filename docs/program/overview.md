# On-Chain Program — Overview

> Native Solana program design, validation model, and security guarantees.

---

## Design Philosophy

The program is written as a **native Solana program** — no Anchor framework, no proc-macro magic. Every account validation is written explicitly by hand in each instruction handler.

This approach:
- Keeps the deployed binary small (no macro expansion overhead).
- Reduces compute units per instruction.
- Makes the validation logic fully auditable with no hidden behavior.
- Removes Anchor, `anchor-lang`, `anchor-spl` from the dependency graph.

The tradeoff is more boilerplate: ownership checks, signer checks, and PDA derivation checks appear explicitly in every handler.

---

## Pinocchio — The Next Step

The current program uses the `solana-program` crate. The recommended upgrade path is to migrate to **[Pinocchio](https://github.com/anza-xyz/pinocchio)**, the zero-dependency, `no_std` Solana program framework developed by Anza that powers [p-token](#p-token--cpi-efficiency).

| Dimension | `solana-program` (current) | `pinocchio` (recommended) |
|-----------|---------------------------|--------------------------|
| `std` dependency | Yes | No (`no_std`) |
| Zero-copy account data | Partial | Full (pointer-based) |
| Binary size | ~130+ KB | ~95 KB (p-token reference) |
| CU overhead | Higher | Significantly lower |
| Heap allocations | Yes | None |

Pinocchio achieves compute savings through zero-copy types — account data is accessed directly via pointers rather than copied into new allocations. This is the same framework used to implement p-token itself.

Migration guide: [Pinocchio Migration](./pinocchio.md)

---

## P-Token & CPI Efficiency

**P-Token** ([SIMD-0266](https://github.com/solana-foundation/solana-improvement-documents/pull/266)) is Solana's compute-optimized replacement for the SPL Token program, built with Pinocchio. It is a **drop-in replacement** — same program address (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`), same account layout, same instruction set.

**Status as of May 2026:**
- ✅ **Devnet: Active** — all devnet token operations now run through p-token
- 🔄 **Mainnet: Targeting May 2026** — governance vote pending

This project **automatically benefits** from p-token: all CPI calls to the SPL Token program (`Split`, `Merge`, `Redeem`, `TokenizePosition`) now execute at p-token compute rates with no code changes required.

### CU savings for this program's token CPIs

| Instruction | SPL Token (old) | p-token | Savings |
|-------------|-----------------|---------|---------|
| `Transfer` (Split/Merge/Redeem) | 4,645 CU | 76 CU | **98%** |
| `TransferChecked` | 6,200 CU | 105 CU | **98%** |
| `MintTo` (TokenizePosition) | 4,538 CU | 119 CU | **97%** |
| `InitializeAccount` (vault ATA) | 4,527 CU | 154 CU | **97%** |

### New `batch` instruction

P-token adds a `batch` instruction that executes multiple token operations in a single CPI call, paying the 1,000 CU base CPI cost only once. This is particularly useful for `TokenizePosition` which currently makes two separate `MintTo` CPIs (one for YES, one for NO):

```rust
// Before (2 separate CPIs, 2 × 1000 CU base cost)
invoke(&mint_to_yes_ix, accounts)?;
invoke(&mint_to_no_ix, accounts)?;

// After p-token batch (1 CPI, 1 × 1000 CU base cost)
invoke(&batch_ix([mint_to_yes_ix, mint_to_no_ix]), accounts)?;
```

See [Pinocchio Migration](./pinocchio.md) for implementation details.

---

## Program Entry Point

```rust
// entrypoint.rs
entrypoint!(process_instruction);
```

Solana calls `process_instruction(program_id, accounts, instruction_data)`. The entry point deserializes the first byte as the instruction discriminant and dispatches to the appropriate handler:

```rust
// processor.rs
match InstructionData::try_from_slice(instruction_data)? {
    InstructionData::CreateMarket(args)    => create_market::process(program_id, accounts, args),
    InstructionData::Split(amount)         => split::process(program_id, accounts, amount),
    InstructionData::Merge(amount)         => merge::process(program_id, accounts, amount),
    InstructionData::PlaceOrder(args)      => place_order::process(program_id, accounts, args),
    InstructionData::CancelOrder(args)     => cancel_order::process(program_id, accounts, args),
    InstructionData::FillOrder(args)       => fill_order::process(program_id, accounts, args),
    InstructionData::ResolveMarket(outcome)=> resolve_market::process(program_id, accounts, outcome),
    InstructionData::Redeem(amount)        => redeem::process(program_id, accounts, amount),
    InstructionData::TokenizePosition(amt) => tokenize_position::process(program_id, accounts, amt),
}
```

---

## Account Validation Pattern

Every handler follows this pattern:

```rust
fn process(program_id: &Pubkey, accounts: &[AccountInfo], args: Args) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user_ai   = next_account_info(iter)?;
    let market_ai = next_account_info(iter)?;
    // ...

    // 1. Signer check
    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    // 2. Ownership check
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    // 3. PDA derivation check
    let (expected_pda, _bump) = find_market_pda(&args.question_hash, program_id);
    if market_ai.key != &expected_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // 4. Deserialize
    let market = Market::try_from_slice(&market_ai.data.borrow())?;

    // 5. Business logic checks
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }

    // ... instruction logic ...

    // 6. Serialize back
    market.serialize(&mut &mut market_ai.data.borrow_mut()[..])?;
    Ok(())
}
```

---

## Instruction Discriminants

Instructions are encoded as a 1-byte enum discriminant followed by Borsh-serialized arguments. This is simpler and smaller than Anchor's 8-byte sha256 discriminant.

| Byte | Instruction |
|------|-------------|
| `0` | `CreateMarket` |
| `1` | `Split` |
| `2` | `Merge` |
| `3` | `PlaceOrder` |
| `4` | `CancelOrder` |
| `5` | `FillOrder` |
| `6` | `ResolveMarket` |
| `7` | `Redeem` |
| `8` | `TokenizePosition` |

---

## Security Model

All security constraints are enforced in the program. Neither the SDK nor the keeper bot enforce business logic — a malicious client cannot bypass on-chain checks.

| Constraint | Where enforced |
|------------|----------------|
| User must sign their own instructions | `is_signer` check on every user-submitted instruction |
| Keeper must sign `FillOrder` | `is_signer` check in `fill_order::process` |
| Only admin can resolve | `user_ai.key == market.admin` in `resolve_market::process` |
| Only order owner can cancel | `user_ai.key == order.user` in `cancel_order::process` |
| Market not already resolved | `!market.resolved` checked before PlaceOrder / Resolve |
| PDA addresses match expected seeds | `find_program_address` derivation check per account |
| Account owned by this program | `account.owner == program_id` for all program accounts |
| No double resolution | `MarketAlreadyResolved` error if `market.resolved == true` |
| No overflow | `checked_add`, `checked_sub`, `checked_mul` everywhere |

---

## Serialization

All accounts use **Borsh** serialization with a **fixed-size layout**. Each account type has a `discriminant` byte at offset 0 (used for `getProgramAccounts` filtering) and a hardcoded `LEN` constant for rent calculations.

There is no variable-length data except `open_orders: [Pubkey; 32]` (fixed 32-slot array).

---

## CPI Usage

The program uses Cross-Program Invocations (CPI) for:

| Operation | Target program | p-token benefit |
|-----------|---------------|-----------------|
| Create Order / UserPosition PDA | `system_program::create_account` | — |
| USDC transfer (Split, Merge, Redeem) | SPL Token → **p-token** on devnet | 98% CU reduction |
| Vault ATA creation (CreateMarket) | SPL Associated Token `create_associated_token_account` | — |
| YES/NO token mint (TokenizePosition) | SPL Token → **p-token** on devnet | 97% CU reduction |

All outbound USDC transfers use `invoke_signed` with the `vault_authority` PDA seeds to authorize the vault account.

---

## Next Steps

- [Pinocchio Migration](./pinocchio.md) — upgrade the program to Pinocchio framework
- [Instructions Reference](./instructions.md)
- [Account Structs](./accounts.md)
- [PDA Seeds](./pda-seeds.md)
