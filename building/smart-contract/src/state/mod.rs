pub mod event;
pub mod global_config;
pub mod market;
pub mod order;
pub mod user_position;

pub use event::Event;
pub use global_config::GlobalConfig;
pub use market::Market;
pub use order::Order;
pub use user_position::UserPosition;

pub const DISCRIMINANT_MARKET:        u8 = 0;
pub const DISCRIMINANT_ORDER:         u8 = 1;
pub const DISCRIMINANT_USER_POSITION: u8 = 2;
pub const DISCRIMINANT_EVENT:         u8 = 3;
pub const DISCRIMINANT_GLOBAL_CONFIG: u8 = 4;

pub const DEFAULT_PUBKEY: [u8; 32] = [0u8; 32];
