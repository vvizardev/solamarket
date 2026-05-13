pub mod market;
pub mod order;
pub mod user_position;

pub use market::Market;
pub use order::Order;
pub use user_position::UserPosition;

// ── discriminants ─────────────────────────────────────────────────────────────
/// Used as the first byte of every program account for `getProgramAccounts` filtering.
pub const DISCRIMINANT_MARKET:        u8 = 0;
pub const DISCRIMINANT_ORDER:         u8 = 1;
pub const DISCRIMINANT_USER_POSITION: u8 = 2;
