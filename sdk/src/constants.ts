import { PublicKey } from "@solana/web3.js";

/**
 * Deployed program ID — update after `solana program deploy`.
 * Placeholder until first deploy.
 */
export const PROGRAM_ID = new PublicKey(
  "11111111111111111111111111111111" // replace with real program ID after deploy
);

export const FILL_FEE_BPS = 5n; // 5 basis points

// Instruction discriminants mirror the Rust InstructionData enum variant order
export const IX = {
  CREATE_MARKET:      0,
  SPLIT:              1,
  MERGE:              2,
  PLACE_ORDER:        3,
  CANCEL_ORDER:       4,
  FILL_ORDER:         5,
  RESOLVE_MARKET:     6,
  REDEEM:             7,
  TOKENIZE_POSITION:  8,
} as const;
