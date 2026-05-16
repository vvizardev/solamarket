# Fee Flow (Current Program)

> How trading fees are computed, split, and credited on-chain today.

Trading economics run entirely inside **`FillOrder`** (instruction 7). No separate fee mint transfers occur during a fill: proceeds and fee shares land in **`UserPosition` fields** (`locked_collateral`, `yes_balance`, `no_balance`) in **collateral atom units** (e.g. mock USDC with 6 decimals).

---

## Diagrams

Plain-text sketches you can read in any editor or Git blame view.

### FillOrder sequence

Credits to keeper and treasury are explicit **`no_balance`** increments after bid/ask positions are updated.

```
   Keeper                          FillOrder (program)
      |                                    |
      |  fill_size + accounts              |
      |----------------------------------->|
      |                                    |
      |                         load Market, compute fees
      |                                    |
      |                         persist_bid_ask_fill
      |                         ----------------------->  Bid UserPosition
      |                         ----------------------->  Ask UserPosition
      |                                    |
      |                         credit_keeper_no_balance
      |                         ----------------------->  Keeper UserPosition (#6)
      |                                    |
      |                         credit_fee_no_balance
      |                         ----------------------->  Fee recipient UserPosition (#7)
      v                                    v
```

### Fee computation and routing

`treasury_share` is whatever remains of `taker_fee` after rebate and keeper slices (integer rounding).

```
                        ┌──► maker_rebate ──► maker UserPosition (persist fill)
   fill_cost ──► taker_fee ──┼──► keeper_reward ─────────────────► UserPosition #6
                        └──► treasury_share ──┐
                                              ├──► UserPosition #7
   fill_cost ──► maker_fee ───────────────────┘

   taker_fee is zero when taker_curve_numer or taker_curve_denom is zero.
```

### Account indices (`FillOrder`)

```
  idx | account
  ----+------------------------------------------
    0 | keeper (signer, writable)
    1 | market
    2 | bid Order PDA
    3 | ask Order PDA
    4 | bid UserPosition
    5 | ask UserPosition
    6 | keeper UserPosition  — receives keeper_reward (no_balance)
    7 | fee_recipient UserPosition — receives treasury_share + maker_fee
```

---

## 1. Where fees are configured

| Source | Role in fee flow |
|--------|------------------|
| **`Market`** (`CreateMarket`) | **Authoritative** for fill economics: `taker_curve_*`, `maker_fee_bps`, `maker_rebate_of_taker_bps`, `keeper_reward_of_taker_bps`, and **`fee_recipient_user`** (treasury owner for protocol-side credits). |
| **`GlobalConfig`** (`Initialize` / `UpdateGlobalConfig`) | Stores `fee_recipient` for protocol-wide bookkeeping. **The current `FillOrder` implementation does not read `GlobalConfig`** — treasury routing uses **`Market.fee_recipient_user` only.** Set `fee_recipient_user` per market to match your intended treasury wallet. |

---

## 2. Trigger and participants

1. A permissionless **keeper** submits **`FillOrder`** with `fill_size`, crossing a **bid** and **ask** on the same **`Market`** (`bid.price ≥ ask.price`).
2. Accounts **4–5**: bid and ask traders’ **`UserPosition`** PDAs.  
3. Account **6**: keeper’s **`UserPosition`** for that market — receives **`keeper_reward`**.  
4. Account **7**: **`UserPosition`** PDA for **`(market, market.fee_recipient_user)`** — receives **`treasury_share + maker_fee`**.

If **`fee_recipient_user`** equals the keeper’s wallet, accounts **#6** and **#7** may be the **same** PDA; credits accumulate on one balance.

---

## 3. Computation order (on-chain)

All intermediate products use wide enough integers (`u128`) before truncating to `u64` where implemented.

1. **Notional (YES leg, collateral priced at bid)**  
   `fill_cost = fill_size × bid.price / 10_000`

2. **Taker fee** (disabled if `taker_curve_numer == 0` or `taker_curve_denom == 0`)  
   `curve = bid.price × (10_000 − bid.price)`  
   `taker_fee = fill_cost × curve × taker_curve_numer / (taker_curve_denom × 10_000 × 10_000)`

3. **Maker fee** (optional flat bps on `fill_cost`)  
   `maker_fee = fill_cost × maker_fee_bps / 10_000`

4. **Split of `taker_fee`**  
   - `maker_rebate = taker_fee × maker_rebate_of_taker_bps / 10_000`  
   - `keeper_reward = taker_fee × keeper_reward_of_taker_bps / 10_000`  
   - `treasury_share = taker_fee − maker_rebate − keeper_reward` (saturating sub)

**Invariant:** `maker_rebate_of_taker_bps + keeper_reward_of_taker_bps ≤ 10_000` should hold at market creation; rounding dust effectively stays with treasury via subtraction order.

---

## 4. Maker vs taker

The **taker** pays **`taker_fee`** (extra collateral debit or reduced sale proceeds). The **maker** may receive **`maker_rebate`** and may pay **`maker_fee`** depending on side.

Determinism (program logic):

1. Compare `bid_order.created_at` vs `ask_order.created_at` (**older = maker**).
2. If equal: lexicographic compare of **bid vs ask order PDA pubkeys** — **smaller pubkey = maker**.

---

## 5. Balance movements (summary)

After **`persist_bid_ask_fill`** updates bid/ask positions and **`credit_*`** runs:

| Recipient | Credit |
|-----------|--------|
| **Maker** | Rebate and fee-adjusted fill economics via bid/ask **`UserPosition`** updates (see tables below). |
| **Taker** | Pays **`taker_fee`** and **`maker_fee`** through the same position updates. |
| **Keeper (`UserPosition` #6)** | `no_balance += keeper_reward` |
| **Treasury (`UserPosition` #7)** | `no_balance += treasury_share + maker_fee` |

### Taker is **bid**, maker is **ask**

| Party | Effect |
|-------|--------|
| Taker bid | `locked_collateral -= fill_cost + taker_fee`; `yes_balance += fill_size` |
| Maker ask | `locked_yes -= fill_size`; `no_balance += fill_cost − maker_fee + maker_rebate` |

### Taker is **ask**, maker is **bid**

| Party | Effect |
|-------|--------|
| Maker bid | `locked_collateral -= fill_cost − maker_rebate`; `yes_balance += fill_size` |
| Taker ask | `locked_yes -= fill_size`; `no_balance += fill_cost − taker_fee − maker_fee` |

---

## 6. What this is *not* (current limitations)

- **No SPL transfer** of USDC to treasury or keeper inside **`FillOrder`** — accrual is internal **`no_balance`** on **`UserPosition`** accounts.
- **`WithdrawFee`** (or similar) to sweep fee balances to an ATA is **not** implemented yet; until then, fee balances participate in the normal collateral / merge semantics documented elsewhere.
- With **`taker_curve_*` disabled**, **`taker_fee == 0`**, so **`keeper_reward`** and **`maker_rebate`** from the taker fee are also **zero**; only **`maker_fee`** (if non-zero) still flows to the treasury position.

---

## 7. Related docs

- [SDK — Fees](../sdk/fees.md) — formulas, diagrams, SDK helpers  
- [Program — Instructions](../program/instructions.md) — `FillOrder` account metas  
- [Keeper — Economics](../keeper/economics.md) — SOL costs vs `keeper_reward`  
