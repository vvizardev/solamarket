import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { IX, PROGRAM_ID } from "./constants";
import {
  CancelOrderArgs,
  CreateMarketArgs,
  FillOrderArgs,
  PlaceOrderArgs,
} from "./types";

// ── encoding helpers ───────────────────────────────────────────────────────

function writeU8(buf: Buffer, offset: number, value: number): number {
  buf.writeUInt8(value, offset);
  return offset + 1;
}

function writeU64(buf: Buffer, offset: number, value: bigint): number {
  buf.writeBigUInt64LE(value, offset);
  return offset + 8;
}

function writeI64(buf: Buffer, offset: number, value: bigint): number {
  buf.writeBigInt64LE(value, offset);
  return offset + 8;
}

function writeBytes(buf: Buffer, offset: number, bytes: Uint8Array): number {
  buf.set(bytes, offset);
  return offset + bytes.length;
}

// ── CreateMarket ───────────────────────────────────────────────────────────

/**
 * Accounts:
 *   0. [writable, signer] admin
 *   1. [writable]         market PDA
 *   2. [writable]         vault ATA
 *   3. []                 vault_authority PDA
 *   4. []                 collateral_mint
 *   5. []                 system_program
 *   6. []                 token_program
 *   7. []                 associated_token_program
 *   8. []                 rent sysvar
 */
export function createMarketInstruction(
  admin:           PublicKey,
  marketPda:       PublicKey,
  vaultAta:        PublicKey,
  vaultAuthority:  PublicKey,
  collateralMint:  PublicKey,
  args:            CreateMarketArgs,
  programId:       PublicKey = PROGRAM_ID,
): TransactionInstruction {
  // discriminant(1) + question_hash(32) + end_time(8) = 41 bytes
  const data   = Buffer.alloc(41);
  let   offset = writeU8(data, 0, IX.CREATE_MARKET);
  offset = writeBytes(data, offset, args.questionHash);
  writeI64(data, offset, args.endTime);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: admin,          isSigner: true,  isWritable: true  },
      { pubkey: marketPda,      isSigner: false, isWritable: true  },
      { pubkey: vaultAta,       isSigner: false, isWritable: true  },
      { pubkey: vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: collateralMint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId,           isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID,                  isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,       isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY,                isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ── Split ──────────────────────────────────────────────────────────────────

/**
 * Accounts:
 *   0. [writable, signer] user
 *   1. [writable]         market PDA
 *   2. [writable]         user_position PDA
 *   3. [writable]         user_usdc_ata
 *   4. [writable]         market vault ATA
 *   5. []                 vault_authority PDA
 *   6. []                 token_program
 *   7. []                 system_program
 */
export function splitInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.SPLIT);
  writeU64(data, offset, amount);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: true  },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: userUsdcAta,     isSigner: false, isWritable: true  },
      { pubkey: vaultAta,        isSigner: false, isWritable: true  },
      { pubkey: vaultAuthority,  isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID,                 isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId,          isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ── Merge ──────────────────────────────────────────────────────────────────

export function mergeInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.MERGE);
  writeU64(data, offset, amount);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: true  },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: userUsdcAta,     isSigner: false, isWritable: true  },
      { pubkey: vaultAta,        isSigner: false, isWritable: true  },
      { pubkey: vaultAuthority,  isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID,                 isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ── PlaceOrder ─────────────────────────────────────────────────────────────

/**
 * Accounts:
 *   0. [writable, signer] user
 *   1. [writable]         market PDA
 *   2. [writable]         user_position PDA
 *   3. [writable]         order PDA (new)
 *   4. []                 system_program
 */
export function placeOrderInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  orderPda:       PublicKey,
  args:           PlaceOrderArgs,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  // discriminant(1) + side(1) + price(8) + size(8) + nonce(8) = 26 bytes
  const data   = Buffer.alloc(26);
  let   offset = writeU8(data, 0, IX.PLACE_ORDER);
  offset = writeU8(data, offset, args.side);
  offset = writeU64(data, offset, args.price);
  offset = writeU64(data, offset, args.size);
  writeU64(data, offset, args.nonce);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: true  },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: orderPda,        isSigner: false, isWritable: true  },
      { pubkey: SystemProgram.programId,          isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ── CancelOrder ────────────────────────────────────────────────────────────

export function cancelOrderInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  orderPda:       PublicKey,
  args:           CancelOrderArgs,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.CANCEL_ORDER);
  writeU64(data, offset, args.nonce);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: false },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: orderPda,        isSigner: false, isWritable: true  },
    ],
    data,
  });
}

// ── FillOrder ──────────────────────────────────────────────────────────────

/**
 * Accounts:
 *   0. [writable, signer] keeper
 *   1. []                 market PDA
 *   2. [writable]         bid_order PDA
 *   3. [writable]         ask_order PDA
 *   4. [writable]         bid_user_position PDA
 *   5. [writable]         ask_user_position PDA
 *   6. [writable]         keeper_user_position PDA
 */
export function fillOrderInstruction(
  keeper:          PublicKey,
  marketPda:       PublicKey,
  bidOrderPda:     PublicKey,
  askOrderPda:     PublicKey,
  bidPositionPda:  PublicKey,
  askPositionPda:  PublicKey,
  keeperPositionPda: PublicKey,
  args:            FillOrderArgs,
  programId:       PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.FILL_ORDER);
  writeU64(data, offset, args.fillSize);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: keeper,             isSigner: true,  isWritable: true  },
      { pubkey: marketPda,          isSigner: false, isWritable: false },
      { pubkey: bidOrderPda,        isSigner: false, isWritable: true  },
      { pubkey: askOrderPda,        isSigner: false, isWritable: true  },
      { pubkey: bidPositionPda,     isSigner: false, isWritable: true  },
      { pubkey: askPositionPda,     isSigner: false, isWritable: true  },
      { pubkey: keeperPositionPda,  isSigner: false, isWritable: true  },
    ],
    data,
  });
}

// ── ResolveMarket ──────────────────────────────────────────────────────────

export function resolveMarketInstruction(
  admin:     PublicKey,
  marketPda: PublicKey,
  outcome:   1 | 2,
  programId: PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data = Buffer.alloc(2);
  writeU8(data, 0, IX.RESOLVE_MARKET);
  writeU8(data, 1, outcome);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: admin,     isSigner: true,  isWritable: false },
      { pubkey: marketPda, isSigner: false, isWritable: true  },
    ],
    data,
  });
}

// ── Redeem ─────────────────────────────────────────────────────────────────

export function redeemInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  userUsdcAta:    PublicKey,
  vaultAta:       PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.REDEEM);
  writeU64(data, offset, amount);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: false },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: userUsdcAta,     isSigner: false, isWritable: true  },
      { pubkey: vaultAta,        isSigner: false, isWritable: true  },
      { pubkey: vaultAuthority,  isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID,                 isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ── TokenizePosition ───────────────────────────────────────────────────────

export function tokenizePositionInstruction(
  user:           PublicKey,
  marketPda:      PublicKey,
  userPositionPda: PublicKey,
  yesMint:        PublicKey,
  noMint:         PublicKey,
  userYesAta:     PublicKey,
  userNoAta:      PublicKey,
  vaultAuthority: PublicKey,
  amount:         bigint,
  programId:      PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const data   = Buffer.alloc(9);
  let   offset = writeU8(data, 0, IX.TOKENIZE_POSITION);
  writeU64(data, offset, amount);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: user,            isSigner: true,  isWritable: true  },
      { pubkey: marketPda,       isSigner: false, isWritable: true  },
      { pubkey: userPositionPda, isSigner: false, isWritable: true  },
      { pubkey: yesMint,         isSigner: false, isWritable: true  },
      { pubkey: noMint,          isSigner: false, isWritable: true  },
      { pubkey: userYesAta,      isSigner: false, isWritable: true  },
      { pubkey: userNoAta,       isSigner: false, isWritable: true  },
      { pubkey: vaultAuthority,  isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID,                 isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId,          isSigner: false, isWritable: false },
    ],
    data,
  });
}
