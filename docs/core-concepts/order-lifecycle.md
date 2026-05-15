# Order Lifecycle

> Full state machine from placement through fill, cancel, and redemption.

---

## Overview

```
User places order
      │
      ▼
PlaceOrder instruction
├─ create Order PDA  [b"order", market, user, nonce.to_le_bytes()]
├─ allocate + init Order account via system_program::create_account
├─ bid:  locked_collateral += size
   ask:  locked_yes        += size
└─ append Order.key to UserPosition.open_orders[]

Keeper detects new Order account (WebSocket / polling)
      │
      ▼
DLOB updated in keeper memory
      │
      ▼  (if best_bid.price >= best_ask.price)
FillOrder instruction (keeper submits)
├─ verify bid.market == ask.market
├─ verify bid.price >= ask.price
├─ compute fill_size = min(bid.remaining, ask.remaining)
├─ bid UserPosition: locked_collateral -= fill_cost; yes_balance += fill_size
├─ ask UserPosition: locked_yes -= fill_size;        no_balance  += fill_proceeds
├─ credit keeper:    no_balance += fill_fee  (5 bps of fill_size)
├─ update Order.fill_amount on both orders
└─ close fully-filled Order accounts → lamports to keeper

      │
      ▼  (if market resolves)
ResolveMarket (admin)
      │
      ▼
Redeem (winner)
├─ debit winning balance
└─ transfer USDC from vault → user ATA
```

---

## 1. PlaceOrder

### What happens on-chain

1. The program validates the user's signature, market state, and price/size parameters.
2. A new `Order` PDA is created and allocated via `system_program::create_account`.
3. The appropriate balance in the user's `UserPosition` is moved from free to locked:
   - Bid: `locked_collateral += size`
   - Ask: `locked_yes += size`
4. The Order's pubkey is appended to `UserPosition.open_orders[]`.

### PlaceOrder args

| Field | Type | Description |
|-------|------|-------------|
| `side` | `u8` | `0` = bid (buy YES), `1` = ask (sell YES) |
| `price` | `u64` | Limit price in basis points (1–9999) |
| `size` | `u64` | Collateral units (6-decimal USDC) |
| `nonce` | `u64` | Client-chosen unique ID for this order |

### Constraints

- Market must not be resolved and `end_time` must not have passed.
- Price must be in range 1–9999.
- Size must be > 0.
- User must have sufficient free balance (`yes_balance` for ask, or enough USDC deposited for bid).
- `open_orders[]` must not be full (max 32 simultaneous open orders).

---

## 2. FillOrder (keeper submits)

### What happens on-chain

The keeper submits both crossing orders' pubkeys. The program:

1. Verifies both orders belong to the same market.
2. Verifies `bid.side == 0` and `ask.side == 1`.
3. Verifies `bid.price >= ask.price` (crossing condition).
4. Computes `fill_size = min(args.fill_size, bid.remaining, ask.remaining)`.
5. Updates both `UserPosition` accounts:

| UserPosition | Change |
|-------------|--------|
| Bid (buyer) | `locked_collateral -= fill_cost`; `yes_balance += fill_size` |
| Ask (seller) | `locked_yes -= fill_size`; `no_balance += (fill_cost - fill_fee)` |
| Keeper | `no_balance += fill_fee` |

6. Updates `Order.fill_amount` on both orders.
7. Closes fully-filled Order accounts, transferring lamports to the keeper's UserPosition account.

### Fill cost formula

```
fill_cost = fill_size × bid.price / 10_000
fill_fee  = fill_size × 5 / 10_000      (5 basis points)
ask_proceeds = fill_cost - fill_fee
```

### Partial fills

If `fill_size < order.size`, the Order account stays open with updated `fill_amount`. The order remains in the DLOB and can be partially filled again in future transactions.

---

## 3. CancelOrder

The order owner can cancel a resting order at any time:

1. Order PDA is closed; rent-exempt lamports return to the **order owner**.
2. Locked balance is released back to free:
   - Bid cancel: `locked_collateral -= order.remaining_size`; implicit collateral returned
   - Ask cancel: `locked_yes -= order.remaining_size`; `yes_balance += order.remaining_size`
3. Order pubkey is removed from `UserPosition.open_orders[]`.

Only the original order owner (the user who placed it) can cancel. Keepers cannot cancel user orders.

---

## 4. ResolveMarket (admin only)

After the question's real-world outcome is known, the admin calls `ResolveMarket(outcome)`:

- `outcome = 1` → YES wins
- `outcome = 2` → NO wins

Constraints: caller must be `market.admin`; market must not already be resolved.

After resolution, no new orders can be placed. Existing unfilled orders should be cancelled.

---

## 5. Redeem (post-resolution)

Winners swap their outcome balance for USDC at 1:1:

```
Redeem(amount)
  if winning_outcome == YES:
    yes_balance -= amount
  else:
    no_balance -= amount
  vault → amount USDC → user's USDC ATA
```

Losers receive nothing for their opposing balance. They can still merge YES+NO pairs before resolution to recover collateral.

---

## Order States Summary

| State | Description |
|-------|-------------|
| Open (resting) | `fill_amount < size`; Order PDA exists; included in DLOB |
| Partially filled | `0 < fill_amount < size`; still resting |
| Fully filled | `fill_amount == size`; Order PDA closed (on fill tx) |
| Cancelled | Order PDA closed (on cancel tx); locked balance released |

---

## Next Steps

- [Resolution](./resolution.md)
- [Instructions — PlaceOrder / FillOrder / CancelOrder](../program/instructions.md#placeorder)
- [Keeper — Overview](../keeper/overview.md)
