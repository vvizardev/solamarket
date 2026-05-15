import { PublicKey } from "@solana/web3.js";
import { Market, Order, UserPosition } from "./types";

// ── helpers ──────────────────────────────────────────────────────────────────

function readPubkey(buf: Buffer, offset: number): PublicKey {
  return new PublicKey(buf.slice(offset, offset + 32));
}

// ── Order (107 bytes) ────────────────────────────────────────────────────────
//
// offset 0   : u8  discriminant (= 1)
// offset 1   : [u8;32] market
// offset 33  : [u8;32] user
// offset 65  : u8  side (0=bid, 1=ask)
// offset 66  : u64 price (LE)
// offset 74  : u64 size (LE)
// offset 82  : u64 fill_amount (LE)
// offset 90  : u64 nonce (LE)
// offset 98  : i64 created_at (LE)
// offset 106 : u8  bump

export function deserializeOrder(data: Buffer): Order {
  if (data.length < 107) {
    throw new Error(`Order account too small: ${data.length} bytes`);
  }
  const discriminant = data.readUInt8(0);
  if (discriminant !== 1) {
    throw new Error(`Expected Order discriminant 1, got ${discriminant}`);
  }
  return {
    discriminant,
    market:     readPubkey(data, 1),
    user:       readPubkey(data, 33),
    side:       data.readUInt8(65) as 0 | 1,
    price:      data.readBigUInt64LE(66),
    size:       data.readBigUInt64LE(74),
    fillAmount: data.readBigUInt64LE(82),
    nonce:      data.readBigUInt64LE(90),
    createdAt:  data.readBigInt64LE(98),
    bump:       data.readUInt8(106),
  };
}

// ── Market (295 bytes) ───────────────────────────────────────────────────────
//
// offset 0   : u8  discriminant (= 0)
// offset 1   : [u8;32] question_hash
// offset 33  : [u8;32] vault
// offset 65  : [u8;32] collateral_mint
// offset 97  : [u8;32] yes_mint
// offset 129 : [u8;32] no_mint
// offset 161 : i64 end_time (LE)
// offset 169 : bool resolved (1 byte)
// offset 170 : u8  winning_outcome
// offset 171 : [u8;32] admin
// offset 203 : u64 order_count (LE)
// offset 211 : [u8;32] event
// offset 243 : u32 taker_curve_numer (LE)
// offset 247 : u32 taker_curve_denom (LE)
// offset 251 : u16 maker_fee_bps (LE)
// offset 253 : u16 maker_rebate_of_taker_bps (LE)
// offset 255 : u16 keeper_reward_of_taker_bps (LE)
// offset 257 : u16 fee_padding (LE)
// offset 259 : [u8;32] fee_recipient_user
// offset 291 : u8  primary_category
// offset 292 : u16 subcategory (LE)
// offset 294 : u8  bump

export function deserializeMarket(data: Buffer): Market {
  if (data.length < 295) {
    throw new Error(`Market account too small: ${data.length} bytes`);
  }
  const discriminant = data.readUInt8(0);
  if (discriminant !== 0) {
    throw new Error(`Expected Market discriminant 0, got ${discriminant}`);
  }
  return {
    discriminant,
    questionHash:            new Uint8Array(data.slice(1, 33)),
    vault:                   readPubkey(data, 33),
    collateralMint:          readPubkey(data, 65),
    yesMint:                 readPubkey(data, 97),
    noMint:                  readPubkey(data, 129),
    endTime:                 data.readBigInt64LE(161),
    resolved:                data.readUInt8(169) !== 0,
    winningOutcome:          data.readUInt8(170),
    admin:                   readPubkey(data, 171),
    orderCount:              data.readBigUInt64LE(203),
    event:                   readPubkey(data, 211),
    takerCurveNumer:         data.readUInt32LE(243),
    takerCurveDenom:         data.readUInt32LE(247),
    makerFeeBps:             data.readUInt16LE(251),
    makerRebateOfTakerBps:   data.readUInt16LE(253),
    keeperRewardOfTakerBps:  data.readUInt16LE(255),
    feePadding:              data.readUInt16LE(257),
    feeRecipientUser:        readPubkey(data, 259),
    primaryCategory:         data.readUInt8(291),
    subcategory:             data.readUInt16LE(292),
    bump:                    data.readUInt8(294),
  };
}

// ── UserPosition (1131 bytes) ─────────────────────────────────────────────────
//
// offset 0    : u8  discriminant (= 2)
// offset 1    : [u8;32] market
// offset 33   : [u8;32] user
// offset 65   : u64 yes_balance (LE)
// offset 73   : u64 no_balance (LE)
// offset 81   : u64 locked_yes (LE)
// offset 89   : u64 locked_no (LE)
// offset 97   : u64 locked_collateral (LE)
// offset 105  : [Pubkey;32] open_orders (32 × 32 = 1024 bytes)
// offset 1129 : u8  open_order_count
// offset 1130 : u8  bump

export function deserializeUserPosition(data: Buffer): UserPosition {
  if (data.length < 1131) {
    throw new Error(`UserPosition account too small: ${data.length} bytes`);
  }
  const discriminant = data.readUInt8(0);
  if (discriminant !== 2) {
    throw new Error(`Expected UserPosition discriminant 2, got ${discriminant}`);
  }
  const openOrderCount = data.readUInt8(1129);
  const openOrders: PublicKey[] = [];
  for (let i = 0; i < openOrderCount; i++) {
    openOrders.push(readPubkey(data, 105 + i * 32));
  }
  return {
    discriminant,
    market:           readPubkey(data, 1),
    user:             readPubkey(data, 33),
    yesBalance:       data.readBigUInt64LE(65),
    noBalance:        data.readBigUInt64LE(73),
    lockedYes:        data.readBigUInt64LE(81),
    lockedNo:         data.readBigUInt64LE(89),
    lockedCollateral: data.readBigUInt64LE(97),
    openOrders,
    openOrderCount,
    bump:             data.readUInt8(1130),
  };
}
