import { PublicKey } from "@solana/web3.js";

// ── on-chain account types ─────────────────────────────────────────────────

export interface Market {
  discriminant:    number;
  questionHash:    Uint8Array;   // 32 bytes
  vault:           PublicKey;
  collateralMint:  PublicKey;
  yesMint:         PublicKey;    // default pubkey until TokenizePosition
  noMint:          PublicKey;
  endTime:         bigint;       // i64
  resolved:        boolean;
  winningOutcome:  number;       // 0=unresolved, 1=YES, 2=NO
  admin:           PublicKey;
  orderCount:      bigint;
  bump:            number;
}

export interface Order {
  discriminant: number;
  market:       PublicKey;
  user:         PublicKey;
  side:         OrderSide;
  price:        bigint;          // basis points 1–9 999
  size:         bigint;
  fillAmount:   bigint;
  nonce:        bigint;
  createdAt:    bigint;
  bump:         number;
}

export interface UserPosition {
  discriminant:      number;
  market:            PublicKey;
  user:              PublicKey;
  yesBalance:        bigint;
  noBalance:         bigint;
  lockedYes:         bigint;
  lockedNo:          bigint;
  lockedCollateral:  bigint;
  openOrders:        PublicKey[];  // up to 32
  openOrderCount:    number;
  bump:              number;
}

// ── instruction argument types ────────────────────────────────────────────

export interface CreateMarketArgs {
  questionHash:  Uint8Array;
  endTime:       bigint;
}

export interface PlaceOrderArgs {
  side:   OrderSide;
  price:  bigint;
  size:   bigint;
  nonce:  bigint;
}

export interface CancelOrderArgs {
  nonce: bigint;
}

export interface FillOrderArgs {
  fillSize: bigint;
}

// ── enums ──────────────────────────────────────────────────────────────────

export enum OrderSide {
  Bid = 0,  // buy YES
  Ask = 1,  // sell YES
}

export enum WinningOutcome {
  Unresolved = 0,
  Yes        = 1,
  No         = 2,
}
