use pinocchio::program_error::ProgramError;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictionMarketError {
    // Account validation (0–9)
    InvalidAccountOwner   = 0,
    InvalidPda            = 1,
    MissingRequiredSigner = 2,
    InvalidDiscriminant   = 3,

    // Market lifecycle (10–19)
    MarketAlreadyResolved = 10,
    MarketExpired         = 11,
    MarketNotResolved     = 12,
    InvalidWinningOutcome = 13,
    NotMarketAdmin        = 14,

    // Order errors (20–29)
    InvalidOrderPrice = 20,
    InvalidOrderSize  = 21,
    InvalidOrderSide  = 22,
    MarketMismatch    = 23,
    NoCrossing        = 24,
    OverFill          = 25,
    NotOrderOwner     = 26,

    // Balance errors (30–39)
    InsufficientBalance = 30,
    Overflow            = 31,
    ZeroAmount          = 32,
    OpenOrdersFull      = 33,

    // Event errors (40–49)
    EventFull            = 40,
    MarketAlreadyInEvent = 41,
    EventAlreadyResolved = 42,
    NotEventAdmin        = 43,
    EventAdminMismatch   = 44,
    InvalidMarketIndex   = 45,
    EventMarketMismatch  = 46,
    NotExclusiveEvent       = 47,

    // Global config (48–49)
    /// UpdateGlobalConfig: signer is not `GlobalConfig::admin`.
    NotGlobalAdmin = 48,
}

impl From<PredictionMarketError> for ProgramError {
    fn from(e: PredictionMarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
