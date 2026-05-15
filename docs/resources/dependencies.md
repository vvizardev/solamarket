# Dependencies

> Rust crates and TypeScript packages used across the monorepo.

---

## Rust (`program/`)

### Current (solana-program)

```toml
# program/Cargo.toml
[dependencies]
solana-program            = "2"
borsh                     = { version = "1", features = ["derive"] }
spl-token                 = { version = "4", features = ["no-entrypoint"] }
spl-associated-token-account = { version = "3", features = ["no-entrypoint"] }
thiserror                 = "1"

[dev-dependencies]
solana-program-test       = "2"
solana-sdk                = "2"
tokio                     = { version = "1", features = ["full"] }
```

| Crate | Purpose |
|-------|---------|
| `solana-program` | Core program SDK: `AccountInfo`, `ProgramResult`, `Pubkey`, `entrypoint!`, `invoke`, `invoke_signed` |
| `borsh` | Deterministic binary serialization for account data and instruction args |
| `spl-token` | SPL Token program CPI calls (`transfer`); `no-entrypoint` feature avoids re-exporting its own entrypoint |
| `spl-associated-token-account` | ATA creation CPI; `no-entrypoint` feature same reason |
| `thiserror` | `#[derive(Error)]` for `PredictionMarketError` — cleaner than manual `Display` impl |
| `solana-program-test` | (dev) Lightweight BanksClient test harness — runs program tests without a validator |
| `solana-sdk` | (dev) `Keypair`, `Transaction`, `Account` types for tests |
| `tokio` | (dev) Async runtime required by `solana-program-test` |

**Not used:** `anchor-lang`, `anchor-spl`, or any Anchor crate.

---

### Recommended Upgrade — Pinocchio + p-token

Migrate the program to **Pinocchio** for maximum CU efficiency. Pinocchio is the `no_std`, zero-allocation framework that powers [p-token](https://solana.com/upgrades/p-token) — Solana's optimized SPL Token replacement, live on devnet since April 2026.

```toml
# program/Cargo.toml — Pinocchio-based (recommended)
[dependencies]
pinocchio                    = "0.7"
pinocchio-pubkey             = "0.2"
pinocchio-token              = "0.4"       # p-token compatible CPI helpers
pinocchio-associated-token   = "0.2"       # ATA CPI helpers
pinocchio-system             = "0.2"       # system_program CPI helpers
borsh                        = { version = "1", features = ["derive"] }

[dev-dependencies]
solana-program               = "2"         # still needed for test harness
solana-program-test          = "2"
solana-sdk                   = "2"
tokio                        = { version = "1", features = ["full"] }
```

| Crate | Purpose | Replaces |
|-------|---------|---------|
| `pinocchio` | Zero-alloc, `no_std` core: `AccountInfo`, `entrypoint!`, `invoke_signed` | `solana-program` |
| `pinocchio-pubkey` | `[u8; 32]` pubkey helpers and macros | `solana-program::pubkey` |
| `pinocchio-token` | p-token / SPL Token CPI instructions (same program address) | `spl-token` |
| `pinocchio-associated-token` | ATA CPI instructions | `spl-associated-token-account` |
| `pinocchio-system` | `create_account`, `transfer` lamports CPI | `solana-program::system_instruction` |
| `borsh` | Account serialization — unchanged | — |

Benefits:
- **~95% CU reduction** on all token CPIs (p-token auto-applies on devnet)
- **Zero heap allocations** in the program binary
- **Smaller binary** (~95 KB vs ~131 KB for spl-token reference)
- **`batch` instruction** support for combining multiple token ops in one CPI

Full migration guide: [Pinocchio Migration](../program/pinocchio.md)

---

## TypeScript (`sdk/`, `keeper/`, `app/`)

```jsonc
// Shared across workspace packages
{
  "@solana/web3.js": "^1",
  "@solana/spl-token": "^0.3",
  "@solana/wallet-adapter-react": "^0.15",
  "@solana/wallet-adapter-wallets": "^0.19",
  "borsh": "^2",
  "next": "^14",
  "react": "^18",
  "tailwindcss": "^3",
  "typescript": "^5"
}
```

| Package | Package | Purpose |
|---------|---------|---------|
| `@solana/web3.js` | `sdk`, `keeper`, `app` | `Connection`, `PublicKey`, `Transaction`, `TransactionInstruction`, `Keypair` |
| `@solana/spl-token` | `sdk` | ATA address derivation (`getAssociatedTokenAddressSync`), token constants (`TOKEN_PROGRAM_ID`) |
| `@solana/wallet-adapter-react` | `app` | React wallet context provider (Phantom, Backpack, Solflare support) |
| `@solana/wallet-adapter-wallets` | `app` | Wallet adapter implementations |
| `borsh` | `sdk` | (Not actually used in current SDK — deserialization is done manually byte-by-byte for full control) |
| `next` | `app` | React framework with file-system routing |
| `tailwindcss` | `app` | Utility-first CSS |
| `typescript` | all | Type safety |

**Not used:** `@coral-xyz/anchor`, `@project-serum/anchor`, or any Anchor TypeScript package.

---

## Monorepo Tooling

| Tool | Version | Purpose |
|------|---------|---------|
| `pnpm` | `^8` | Fast workspace-aware package manager |
| `ts-node` | `^10` | Run TypeScript scripts directly |
| `jest` | `^29` | Unit tests for DLOB logic |
| `@types/node` | `^20` | Node.js type definitions |

---

## Solana CLI

```bash
# Install
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify
solana --version
# solana-cli 2.x.x

# cargo-build-sbf is included with the Solana CLI install
cargo build-sbf --version
```

Required version: Solana CLI **2.x** (matching `solana-program = "2"` in Cargo.toml).

---

## Version Pinning

Solana's crates are tightly coupled — `solana-program`, `solana-program-test`, and `solana-sdk` must all use the same major version. If you see linker or ABI errors, verify all three are pinned to the same version in `Cargo.lock`.

```toml
# Workspace Cargo.toml — pin all Solana crates to same version
[workspace.dependencies]
solana-program      = "=2.1.0"
solana-program-test = "=2.1.0"
solana-sdk          = "=2.1.0"
```
