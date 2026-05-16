# Outcome Tokens

> Split, merge, and opt-in SPL tokenization of YES/NO positions.

---

## Split — Deposit USDC for YES + NO Shares

`Split(amount)` is the entry point for all trading. You deposit USDC into the market vault and receive equal amounts of YES and NO internal balance.

```typescript
import { splitInstruction } from "@solamarket/sdk";

const ix = splitInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  userUsdcAta,
  vaultAta,
  vaultAuthority,
  100_000_000n,  // 100 USDC
  PROGRAM_ID,
);
```

After `Split(100 USDC)`:
- `UserPosition.yes_balance += 100_000_000`
- `UserPosition.no_balance  += 100_000_000`
- Vault USDC balance += 100 USDC

You can now sell YES shares (place an ask order) or buy additional YES (place a bid using existing no_balance converted via Split).

---

## Merge — Withdraw USDC by Burning YES + NO Shares

`Merge(amount)` is the reverse of Split. It burns equal YES and NO balances and returns the collateral:

```typescript
import { mergeInstruction } from "@solamarket/sdk";

const ix = mergeInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  userUsdcAta,
  vaultAta,
  vaultAuthority,
  50_000_000n,   // 50 USDC worth
  PROGRAM_ID,
);
```

After `Merge(50 USDC)`:
- `UserPosition.yes_balance -= 50_000_000`
- `UserPosition.no_balance  -= 50_000_000`
- User receives 50 USDC back from vault

`Merge` is available any time — before and after market resolution. It is the only way for the losing side to recover any collateral (by holding a matched pair of YES + NO, then merging before or after resolution).

---

## Redeem — Claim Winnings (Post-Resolution)

After a market resolves, the winning side can redeem at 1:1:

```typescript
import { redeemInstruction } from "@solamarket/sdk";

// Check the resolved outcome first
const market = await fetchMarket(connection, marketPda);
if (!market.resolved) throw new Error("Market not resolved yet");

const isYesWinner = market.winningOutcome === 1;
const position = await fetchUserPosition(connection, userPositionPda);
const redeemAmount = isYesWinner ? position.yesBalance : position.noBalance;

const ix = redeemInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  userUsdcAta,
  vaultAta,
  vaultAuthority,
  redeemAmount,
  PROGRAM_ID,
);
```

---

## TokenizePosition — Opt-In SPL Token Minting

By default, YES and NO balances are internal numbers in `UserPosition`. To use them in external DeFi (LP pools, lending protocols, bridges), call `TokenizePosition` to mint real SPL tokens.

```typescript
import { tokenizePositionInstruction } from "@solamarket/sdk";

const ix = tokenizePositionInstruction(
  user.publicKey,
  marketPda,
  userPositionPda,
  yesMint,           // market.yesMint (Pubkey.default until first tokenization)
  noMint,            // market.noMint
  userYesAta,        // user's YES token ATA
  userNoAta,         // user's NO token ATA
  yesAuthority,      // PDA controlling YES mint
  noAuthority,       // PDA controlling NO mint
  50_000_000n,       // amount to tokenize (USDC units)
  PROGRAM_ID,
);
```

**What this does:**
1. Lazily creates the YES/NO SPL mints for this market (if not yet created).
2. Creates user YES ATA + user NO ATA (user pays ~0.004 SOL in rent).
3. Mints `amount` YES tokens and `amount` NO tokens to the user's ATAs.
4. Deducts `amount` from `UserPosition.yes_balance` and `no_balance`.

**Cost:** ~0.004 SOL (two ATAs at ~0.002 SOL each), one-time per market per user.

**After tokenization:** The minted SPL tokens can be transferred, traded in AMMs, or used as collateral in lending protocols. They are standard SPL tokens with no special lock-up.

---

## YES/NO Token Economics

When you tokenize:
- 1 YES token + 1 NO token always = 1 USDC (they represent the two sides of the same bet).
- If the market resolves YES: each YES token redeems for 1 USDC; NO tokens are worthless.
- If the market resolves NO: each NO token redeems for 1 USDC; YES tokens are worthless.

This is identical to Polymarket's CTF (Conditional Token Framework) economics, but implemented directly in the Solana program without the full CTF contract.

---

## P-Token & CU Efficiency

All token operations in `Split`, `Merge`, `Redeem`, and `TokenizePosition` call the SPL Token program via CPI. Since **p-token is live on Solana devnet** (activated April 2026), these calls now run at dramatically lower compute costs:

| Operation | CPI | Before p-token | After p-token |
|-----------|-----|----------------|---------------|
| Split (USDC deposit) | `Transfer` | 4,645 CU | 76 CU |
| Merge (USDC withdrawal) | `Transfer` | 4,645 CU | 76 CU |
| Redeem (USDC withdrawal) | `Transfer` | 4,645 CU | 76 CU |
| TokenizePosition (mint YES) | `MintTo` | 4,538 CU | 119 CU |
| TokenizePosition (mint NO) | `MintTo` | 4,538 CU | 119 CU |

No SDK or client changes are needed — p-token is transparent to callers.

### `batch` instruction opportunity for `TokenizePosition`

P-token introduces a new `batch` instruction that executes multiple token operations in a single CPI, paying the 1,000 CU base CPI cost only once. `TokenizePosition` currently makes **two separate `MintTo` CPIs** (YES + NO), paying 1,000 CU overhead twice.

After the program is [migrated to Pinocchio](../program/pinocchio.md), `TokenizePosition` can use `batch` to save ~1,000 CU per call:

```rust
// Two MintTo ops in one batch CPI instead of two separate invoke() calls
Batch {
    instructions: &[mint_yes_ix, mint_no_ix],
}.invoke()?;
```

---

## Comparison with Polymarket CTF Tokens

| Dimension | Polymarket CTF | This project |
|-----------|---------------|--------------|
| Token standard | ERC-1155 (Polygon) | SPL (Solana) |
| Default path | Tokens required for all trading | Internal balances (no tokens needed) |
| Tokenization cost | Paid upfront | Free default; ~0.004 SOL opt-in |
| Composability | EVM DeFi | Solana DeFi |
| Redemption | Exchange contract | Program `Redeem` instruction |

---

## Next Steps

- [WebSocket](./websocket.md)
- [Core Concepts — Positions & Tokens](../core-concepts/positions-and-tokens.md)
- [Instructions — Split / Merge / TokenizePosition](../program/instructions.md#split)
