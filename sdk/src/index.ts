// ── constants ──────────────────────────────────────────────────────────────
export { PROGRAM_ID, FILL_FEE_BPS, IX } from "./constants";

// ── types ──────────────────────────────────────────────────────────────────
export type {
  Market,
  Order,
  UserPosition,
  CreateMarketArgs,
  PlaceOrderArgs,
  CancelOrderArgs,
  FillOrderArgs,
} from "./types";
export { OrderSide, WinningOutcome } from "./types";

// ── PDA helpers ────────────────────────────────────────────────────────────
export {
  findMarketPda,
  findVaultAuthorityPda,
  findOrderPda,
  findUserPositionPda,
} from "./pda";

// ── account deserializers + fetchers ──────────────────────────────────────
export {
  deserializeMarket,
  deserializeOrder,
  deserializeUserPosition,
  fetchMarket,
  fetchOrder,
  fetchUserPosition,
  fetchOrdersForMarket,
} from "./accounts";

// ── instruction builders ──────────────────────────────────────────────────
export {
  createMarketInstruction,
  splitInstruction,
  mergeInstruction,
  placeOrderInstruction,
  cancelOrderInstruction,
  fillOrderInstruction,
  resolveMarketInstruction,
  redeemInstruction,
  tokenizePositionInstruction,
} from "./instructions";

// ── DLOB ───────────────────────────────────────────────────────────────────
export { DLOB }            from "./dlob/DLOB";
export { DLOBNode }        from "./dlob/DLOBNode";
export { OrderSubscriber } from "./dlob/OrderSubscriber";
export type { OrderUpdateCallback } from "./dlob/OrderSubscriber";

// ── utils ──────────────────────────────────────────────────────────────────
export { computeFillCost, computeFillFee, clamp, hashQuestion } from "./utils/math";
