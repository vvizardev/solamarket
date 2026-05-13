import { Connection, PublicKey } from "@solana/web3.js";
import { Market, Order, UserPosition } from "./types";
import { PROGRAM_ID } from "./constants";

// ── raw byte deserializers ─────────────────────────────────────────────────
// Mirrors Borsh field layout from the Rust structs byte-for-byte.
// Borsh uses little-endian for all integer types.

function readPubkey(buf: Buffer, offset: number): [PublicKey, number] {
  const key = new PublicKey(buf.subarray(offset, offset + 32));
  return [key, offset + 32];
}

function readU64(buf: Buffer, offset: number): [bigint, number] {
  const value = buf.readBigUInt64LE(offset);
  return [value, offset + 8];
}

function readI64(buf: Buffer, offset: number): [bigint, number] {
  const value = buf.readBigInt64LE(offset);
  return [value, offset + 8];
}

// ── Market ─────────────────────────────────────────────────────────────────

/**
 * Deserialize a Market account (212 bytes).
 * Layout:
 *   0        discriminant  u8
 *   1..32    question_hash [u8;32]
 *   33..64   vault         Pubkey
 *   65..96   collateral_mint Pubkey
 *   97..128  yes_mint      Pubkey
 *   129..160 no_mint       Pubkey
 *   161..168 end_time      i64
 *   169      resolved      bool
 *   170      winning_outcome u8
 *   171..202 admin         Pubkey
 *   203..210 order_count   u64
 *   211      bump          u8
 */
export function deserializeMarket(data: Buffer): Market {
  let o = 0;

  const discriminant = data.readUInt8(o++);
  const questionHash = new Uint8Array(data.subarray(o, o + 32));
  o += 32;

  let vault: PublicKey;
  [vault, o] = readPubkey(data, o);
  let collateralMint: PublicKey;
  [collateralMint, o] = readPubkey(data, o);
  let yesMint: PublicKey;
  [yesMint, o] = readPubkey(data, o);
  let noMint: PublicKey;
  [noMint, o] = readPubkey(data, o);

  let endTime: bigint;
  [endTime, o] = readI64(data, o);

  const resolved        = data.readUInt8(o++) !== 0;
  const winningOutcome  = data.readUInt8(o++);

  let admin: PublicKey;
  [admin, o] = readPubkey(data, o);

  let orderCount: bigint;
  [orderCount, o] = readU64(data, o);

  const bump = data.readUInt8(o);

  return {
    discriminant,
    questionHash,
    vault,
    collateralMint,
    yesMint,
    noMint,
    endTime,
    resolved,
    winningOutcome,
    admin,
    orderCount,
    bump,
  };
}

// ── Order ──────────────────────────────────────────────────────────────────

/**
 * Deserialize an Order account (107 bytes).
 * Layout:
 *   0        discriminant  u8
 *   1..32    market        Pubkey
 *   33..64   user          Pubkey
 *   65       side          u8
 *   66..73   price         u64
 *   74..81   size          u64
 *   82..89   fill_amount   u64
 *   90..97   nonce         u64
 *   98..105  created_at    i64
 *   106      bump          u8
 */
export function deserializeOrder(data: Buffer): Order {
  let o = 0;

  const discriminant = data.readUInt8(o++);

  let market: PublicKey;
  [market, o] = readPubkey(data, o);
  let user: PublicKey;
  [user, o] = readPubkey(data, o);

  const side = data.readUInt8(o++) as 0 | 1;

  let price: bigint;
  [price, o] = readU64(data, o);
  let size: bigint;
  [size, o] = readU64(data, o);
  let fillAmount: bigint;
  [fillAmount, o] = readU64(data, o);
  let nonce: bigint;
  [nonce, o] = readU64(data, o);
  let createdAt: bigint;
  [createdAt, o] = readI64(data, o);

  const bump = data.readUInt8(o);

  return { discriminant, market, user, side, price, size, fillAmount, nonce, createdAt, bump };
}

// ── UserPosition ───────────────────────────────────────────────────────────

/**
 * Deserialize a UserPosition account (1131 bytes).
 */
export function deserializeUserPosition(data: Buffer): UserPosition {
  let o = 0;

  const discriminant = data.readUInt8(o++);

  let market: PublicKey;
  [market, o] = readPubkey(data, o);
  let user: PublicKey;
  [user, o] = readPubkey(data, o);

  let yesBalance: bigint;
  [yesBalance, o] = readU64(data, o);
  let noBalance: bigint;
  [noBalance, o] = readU64(data, o);
  let lockedYes: bigint;
  [lockedYes, o] = readU64(data, o);
  let lockedNo: bigint;
  [lockedNo, o] = readU64(data, o);
  let lockedCollateral: bigint;
  [lockedCollateral, o] = readU64(data, o);

  const openOrders: PublicKey[] = [];
  for (let i = 0; i < 32; i++) {
    let pk: PublicKey;
    [pk, o] = readPubkey(data, o);
    openOrders.push(pk);
  }

  const openOrderCount = data.readUInt8(o++);
  const bump           = data.readUInt8(o);

  return {
    discriminant,
    market,
    user,
    yesBalance,
    noBalance,
    lockedYes,
    lockedNo,
    lockedCollateral,
    openOrders: openOrders.slice(0, openOrderCount),
    openOrderCount,
    bump,
  };
}

// ── fetchers ───────────────────────────────────────────────────────────────

export async function fetchMarket(
  connection: Connection,
  marketPubkey: PublicKey,
): Promise<Market> {
  const ai = await connection.getAccountInfo(marketPubkey);
  if (!ai) throw new Error(`Market account ${marketPubkey.toBase58()} not found`);
  return deserializeMarket(Buffer.from(ai.data));
}

export async function fetchOrder(
  connection: Connection,
  orderPubkey: PublicKey,
): Promise<Order> {
  const ai = await connection.getAccountInfo(orderPubkey);
  if (!ai) throw new Error(`Order account ${orderPubkey.toBase58()} not found`);
  return deserializeOrder(Buffer.from(ai.data));
}

export async function fetchUserPosition(
  connection: Connection,
  positionPubkey: PublicKey,
): Promise<UserPosition> {
  const ai = await connection.getAccountInfo(positionPubkey);
  if (!ai) throw new Error(`UserPosition account ${positionPubkey.toBase58()} not found`);
  return deserializeUserPosition(Buffer.from(ai.data));
}

/** Fetch all Order accounts for a given market using `getProgramAccounts` + memcmp. */
export async function fetchOrdersForMarket(
  connection:   Connection,
  marketPubkey: PublicKey,
  programId:    PublicKey = PROGRAM_ID,
): Promise<{ pubkey: PublicKey; order: Order }[]> {
  const accounts = await connection.getProgramAccounts(programId, {
    filters: [
      { memcmp: { offset: 0, bytes: Buffer.from([1]).toString("base64") } }, // discriminant = Order
      { memcmp: { offset: 1, bytes: marketPubkey.toBase58() } },              // market at offset 1
    ],
  });

  return accounts.map(({ pubkey, account }) => ({
    pubkey,
    order: deserializeOrder(Buffer.from(account.data)),
  }));
}
