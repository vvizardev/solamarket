/** Compute fill cost for a bid: collateral = size * price / 10_000 */
export function computeFillCost(size: bigint, price: bigint): bigint {
  return (size * price) / 10_000n;
}

/** Compute keeper fill fee: fee = size * 5 / 10_000 (5 bps) */
export function computeFillFee(size: bigint): bigint {
  return (size * 5n) / 10_000n;
}

/** Clamp a value to [min, max]. */
export function clamp(value: bigint, min: bigint, max: bigint): bigint {
  if (value < min) return min;
  if (value > max) return max;
  return value;
}

/**
 * Convert a question string to its SHA-256 hash (32 bytes).
 * Used as the Market PDA seed.
 */
export async function hashQuestion(question: string): Promise<Uint8Array> {
  const encoded = new TextEncoder().encode(question);
  const digest  = await crypto.subtle.digest("SHA-256", encoded);
  return new Uint8Array(digest);
}
