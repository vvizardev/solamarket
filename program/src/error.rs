use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum PredictionMarketError {
    // ── account-validation ──────────────────────────────────────────────
    #[error("unexpected account owner")]
    InvalidAccountOwner = 0,

    #[error("PDA derivation mismatch")]
    InvalidPda = 1,

    #[error("missing required signer")]
    MissingRequiredSigner = 2,

    #[error("account discriminant mismatch")]
    InvalidDiscriminant = 3,

    // ── market lifecycle ─────────────────────────────────────────────────
    #[error("market is already resolved")]
    MarketAlreadyResolved = 10,

    #[error("market trading window has closed")]
    MarketExpired = 11,

    #[error("market is not yet resolved")]
    MarketNotResolved = 12,

    #[error("invalid winning outcome (must be 1=YES or 2=NO)")]
    InvalidWinningOutcome = 13,

    #[error("caller is not the market admin")]
    NotMarketAdmin = 14,

    // ── order ─────────────────────────────────────────────────────────────
    #[error("order price must be 1–9 999 basis points")]
    InvalidOrderPrice = 20,

    #[error("order size must be greater than zero")]
    InvalidOrderSize = 21,

    #[error("bid side must be 0, ask side must be 1")]
    InvalidOrderSide = 22,

    #[error("orders must belong to the same market")]
    MarketMismatch = 23,

    #[error("bid price is below ask price — no crossing")]
    NoCrossing = 24,

    #[error("fill would exceed remaining order size")]
    OverFill = 25,

    #[error("caller is not the order owner")]
    NotOrderOwner = 26,

    // ── balances ──────────────────────────────────────────────────────────
    #[error("insufficient balance")]
    InsufficientBalance = 30,

    #[error("arithmetic overflow")]
    Overflow = 31,

    #[error("amount must be greater than zero")]
    ZeroAmount = 32,

    #[error("user position open_orders list is full (max 32)")]
    OpenOrdersFull = 33,
}

impl From<PredictionMarketError> for ProgramError {
    fn from(e: PredictionMarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
