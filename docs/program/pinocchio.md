# Pinocchio Migration Guide

> Upgrade the prediction-market program from `solana-program` to the Pinocchio framework for maximum CU efficiency.

---

## What Is Pinocchio?

[Pinocchio](https://github.com/anza-xyz/pinocchio) is a zero-dependency, `no_std` Solana program framework developed by Anza (the core Solana engineering team). It is the framework that powers **p-token** — Solana's compute-optimized SPL Token replacement ([SIMD-0266](https://github.com/solana-foundation/solana-improvement-documents/pull/266)).

The "p" in both p-token and Pinocchio are the same: the framework was purpose-built to demonstrate what native Solana programs can achieve when stripped of all unnecessary allocations.

### Why migrate?

| Dimension | `solana-program` | `pinocchio` |
|-----------|-----------------|-------------|
| Heap allocations | Yes (`Box`, `Vec` in internals) | Zero |
| Zero-copy account data | Partial | Full (raw pointer slices) |
| `std` dependency | Yes | `no_std` |
| Binary size (p-token reference) | ~131 KB | ~95 KB |
| CU per `Pubkey` comparison | Higher | Lower (direct byte comparison) |
| CU per account deserialization | Higher (alloc + copy) | Lower (zero-copy) |

For a program as compute-intensive as a DEX / order book, migrating to Pinocchio can meaningfully reduce per-instruction CU cost — especially on hot paths like `PlaceOrder` and `FillOrder`.

---

## Dependency Changes

```toml
# program/Cargo.toml — BEFORE
[dependencies]
solana-program = "2"
borsh          = { version = "1", features = ["derive"] }
spl-token      = { version = "4", features = ["no-entrypoint"] }
spl-associated-token-account = { version = "3", features = ["no-entrypoint"] }
thiserror      = "1"
```

```toml
# program/Cargo.toml — AFTER (Pinocchio)
[dependencies]
pinocchio                    = "0.7"
pinocchio-pubkey             = "0.2"
pinocchio-token              = "0.4"        # p-token CPI helpers
pinocchio-associated-token   = "0.2"        # ATA CPI helpers
pinocchio-system             = "0.2"        # system_program CPI helpers
borsh                        = { version = "1", features = ["derive"] }

# No thiserror — pinocchio programs use ProgramError directly
```

> **Note:** `solana-program` is still required as a **dev-dependency** for `solana-program-test` in tests.

```toml
[dev-dependencies]
solana-program      = "2"
solana-program-test = "2"
solana-sdk          = "2"
tokio               = { version = "1", features = ["full"] }
```

---

## Entrypoint

Pinocchio provides its own `entrypoint!` macro:

```rust
// entrypoint.rs — BEFORE (solana-program)
use solana_program::entrypoint;
entrypoint!(process_instruction);
```

```rust
// entrypoint.rs — AFTER (pinocchio)
use pinocchio::entrypoint;
entrypoint!(process_instruction);
```

The function signature is identical.

---

## AccountInfo

Pinocchio's `AccountInfo` is a zero-copy view over the raw account memory, not a deserialized struct:

```rust
// BEFORE (solana-program)
use solana_program::account_info::{AccountInfo, next_account_info};

fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user_ai = next_account_info(iter)?;
    if !user_ai.is_signer { ... }
    if user_ai.owner != program_id { ... }
    let lamports = **user_ai.lamports.borrow();
}
```

```rust
// AFTER (pinocchio)
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

fn process(program_id: &[u8; 32], accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if !user_ai.is_signer() { ... }         // method call, not field
    if user_ai.owner() != program_id { ... } // returns &[u8; 32]
    let lamports = user_ai.lamports();       // u64 direct
}
```

Key API differences:

| `solana-program` | `pinocchio` |
|-----------------|-------------|
| `ai.is_signer` (field) | `ai.is_signer()` (method) |
| `*ai.lamports.borrow()` | `ai.lamports()` |
| `ai.owner` (field, `&Pubkey`) | `ai.owner()` (method, `&[u8; 32]`) |
| `ai.key` (field) | `ai.key()` (method) |
| `ai.data.borrow()` | `ai.try_borrow_data()?` |
| `ai.data.borrow_mut()` | `ai.try_borrow_mut_data()?` |

---

## Pubkey

Pinocchio represents pubkeys as `[u8; 32]` directly, not a newtype:

```rust
// BEFORE
use solana_program::pubkey::Pubkey;
let expected = Pubkey::find_program_address(&[b"market", hash], program_id);
if market_ai.key != &expected.0 { ... }

// AFTER
use pinocchio_pubkey::pubkey;
// Use find_program_address from solana-program in build scripts, or
// store canonical seeds and check manually via byte comparison
if market_ai.key() != expected_pda { ... }  // both &[u8; 32]
```

---

## Token CPIs (p-token Compatible)

The `pinocchio-token` crate provides zero-allocation CPI helpers that target the same program address as SPL Token (and therefore p-token):

```rust
// BEFORE (spl-token CPI)
use spl_token::instruction::transfer;
use solana_program::program::invoke_signed;

invoke_signed(
    &transfer(
        &spl_token::ID,
        source.key, dest.key, authority.key,
        &[], amount,
    )?,
    &[source.clone(), dest.clone(), authority.clone(), token_program.clone()],
    &[&[SEED_VAULT_AUTHORITY, market.key.as_ref(), &[bump]]],
)?;
```

```rust
// AFTER (pinocchio-token CPI — works with both SPL Token and p-token)
use pinocchio_token::instructions::Transfer;

Transfer {
    from:      source,
    to:        dest,
    authority: vault_authority,
    amount,
}.invoke_signed(&[&[SEED_VAULT_AUTHORITY, market.key(), &[bump]]])?;
```

The `pinocchio-token` helpers automatically target the canonical SPL Token program address — which is also the p-token address after the feature gate activates. No address change needed.

### Batch instruction (p-token only)

P-token's new `batch` instruction lets you combine multiple token operations into a single CPI, saving the 1,000 CU base cost for each additional call:

```rust
use pinocchio_token::instructions::{Batch, MintTo};

// TokenizePosition: mint YES + NO in one CPI instead of two
Batch {
    instructions: &[
        MintTo { mint: yes_mint, to: user_yes_ata, authority: yes_authority, amount }.into(),
        MintTo { mint: no_mint,  to: user_no_ata,  authority: no_authority,  amount }.into(),
    ],
}.invoke()?;
```

This saves ~1,000 CU per batched call. For `TokenizePosition` (2 mints), that is 1,000 CU saved immediately.

---

## Error Handling

Pinocchio uses `ProgramError` directly. Drop `thiserror` and implement `From` manually or use integer casts:

```rust
// BEFORE (thiserror)
#[derive(Debug, thiserror::Error)]
pub enum PredictionMarketError {
    #[error("market already resolved")]
    MarketAlreadyResolved = 10,
}
impl From<PredictionMarketError> for ProgramError {
    fn from(e: PredictionMarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

```rust
// AFTER (pinocchio, no thiserror)
use pinocchio::program_error::ProgramError;

#[repr(u32)]
pub enum PredictionMarketError {
    InvalidAccountOwner   = 0,
    // ... same variants ...
    MarketAlreadyResolved = 10,
}

impl From<PredictionMarketError> for ProgramError {
    fn from(e: PredictionMarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

---

## Serialization

Pinocchio has no built-in serialization. Continue using `borsh` for account data — the dependency is unchanged. The key difference is using zero-copy reads where possible instead of full deserialization for hot-path checks:

```rust
// Zero-copy discriminant check (no full deserialization)
let discriminant = unsafe { *market_ai.try_borrow_data()?.get_unchecked(0) };
if discriminant != DISCRIMINANT_MARKET {
    return Err(PredictionMarketError::InvalidDiscriminant.into());
}

// Full deserialization only when needed
let market = Market::try_from_slice(&market_ai.try_borrow_data()?)?;
```

---

## Migration Checklist

- [ ] Replace `solana-program` with `pinocchio`, `pinocchio-pubkey`, `pinocchio-token`, `pinocchio-system`, `pinocchio-associated-token` in `Cargo.toml`
- [ ] Update `entrypoint.rs` to use `pinocchio::entrypoint`
- [ ] Update all `AccountInfo` field accesses to method calls (`.is_signer()`, `.owner()`, `.key()`, etc.)
- [ ] Replace `spl_token::instruction::*` CPI calls with `pinocchio_token::instructions::*`
- [ ] Replace `system_instruction::create_account` with `pinocchio_system::instructions::CreateAccount`
- [ ] Replace `spl_associated_token_account::instruction::*` with `pinocchio_associated_token::instructions::*`
- [ ] Remove `thiserror` from dependencies; use `#[repr(u32)]` enum directly
- [ ] Add `pinocchio-token::instructions::Batch` for `TokenizePosition` (2 mints → 1 CPI)
- [ ] Keep `solana-program`, `solana-program-test`, `solana-sdk` in `[dev-dependencies]` for tests

---

## Further Reading

- [Pinocchio repository](https://github.com/anza-xyz/pinocchio)
- [p-token repository](https://github.com/solana-program/token) — real-world Pinocchio program
- [Helius: Solana P-Token deep dive](https://www.helius.dev/blog/solana-p-token)
- [SIMD-0266: Efficient Token Program](https://github.com/solana-foundation/solana-improvement-documents/pull/266)
- [P-Token on Solana.com](https://solana.com/upgrades/p-token)
