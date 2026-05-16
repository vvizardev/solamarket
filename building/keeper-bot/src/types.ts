import { PublicKey } from "@solana/web3.js";

/**
 * On-chain Market account (295 bytes).
 * Byte-exact match of the Rust `Market` struct in state/market.rs.
 */
export interface Market {
  discriminant: number;
  questionHash: Uint8Array;    // 32 bytes — SHA-256 of the question string
  vault: PublicKey;
  collateralMint: PublicKey;
  yesMint: PublicKey;
  noMint: PublicKey;
  endTime: bigint;             // i64 Unix timestamp
  resolved: boolean;
  winningOutcome: number;      // 0=unresolved, 1=YES, 2=NO
  admin: PublicKey;
  orderCount: bigint;
  event: PublicKey;            // Pubkey::default() for standalone markets

  // Fee schedule (Polymarket-style taker curve)
  takerCurveNumer: number;
  takerCurveDenom: number;
  makerFeeBps: number;
  makerRebateOfTakerBps: number;
  keeperRewardOfTakerBps: number;
  feePadding: number;
  feeRecipientUser: PublicKey; // treasury owner; must be non-default in production

  primaryCategory: number;
  subcategory: number;
  bump: number;
}

/**
 * On-chain Order account (107 bytes).
 * Byte-exact match of the Rust `Order` struct in state/order.rs.
 */
export interface Order {
  discriminant: number;
  market: PublicKey;
  user: PublicKey;
  side: 0 | 1;      // 0 = bid (buy YES), 1 = ask (sell YES)
  price: bigint;    // basis points 1–9999
  size: bigint;     // original collateral units
  fillAmount: bigint;
  nonce: bigint;
  createdAt: bigint; // i64 Unix timestamp
  bump: number;
}

/**
 * On-chain UserPosition account (1131 bytes).
 * Byte-exact match of the Rust `UserPosition` struct in state/user_position.rs.
 */
export interface UserPosition {
  discriminant: number;
  market: PublicKey;
  user: PublicKey;
  yesBalance: bigint;
  noBalance: bigint;
  lockedYes: bigint;
  lockedNo: bigint;
  lockedCollateral: bigint;
  openOrders: PublicKey[];   // sliced to openOrderCount
  openOrderCount: number;
  bump: number;
}

/** Account discriminants */
export const DISCRIMINANT_MARKET: number        = 0;
export const DISCRIMINANT_ORDER: number         = 1;
export const DISCRIMINANT_USER_POSITION: number = 2;
export const DISCRIMINANT_EVENT: number         = 3;

/** FillOrder instruction discriminant (index in processor.rs dispatch table) */
export const FILL_ORDER_DISCRIMINANT: number = 7;
