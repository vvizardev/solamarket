use borsh::{BorshDeserialize, BorshSerialize};

// ── argument structs ─────────────────────────────────────────────────────────

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateMarketArgs {
    /// SHA-256 hash of the question string.
    pub question_hash: [u8; 32],
    /// Unix timestamp after which no new orders are accepted.
    pub end_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PlaceOrderArgs {
    /// 0 = bid (buy YES), 1 = ask (sell YES).
    pub side: u8,
    /// Price in basis points, 1–9 999.
    pub price: u64,
    /// Collateral size (USDC with 6-decimal precision).
    pub size: u64,
    /// Monotonically increasing nonce scoped to (user, market).
    pub nonce: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CancelOrderArgs {
    pub nonce: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct FillOrderArgs {
    /// Desired fill size; program caps it at min(bid.remaining, ask.remaining).
    pub fill_size: u64,
}

// ── top-level enum ────────────────────────────────────────────────────────────

/// Borsh-serialized; first byte is the variant discriminant.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum InstructionData {
    CreateMarket(CreateMarketArgs), // 0
    Split(u64),                     // 1 – deposit `amount` USDC → mint YES+NO
    Merge(u64),                     // 2 – burn YES+NO → withdraw `amount` USDC
    PlaceOrder(PlaceOrderArgs),     // 3
    CancelOrder(CancelOrderArgs),   // 4
    FillOrder(FillOrderArgs),       // 5
    ResolveMarket(u8),              // 6 – outcome: 1=YES, 2=NO
    Redeem(u64),                    // 7 – redeem `amount` winning tokens → USDC
    TokenizePosition(u64),          // 8 – mint real SPL YES/NO tokens (opt-in)
}
