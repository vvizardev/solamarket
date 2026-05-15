use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

/// On-chain Event account — 592 bytes.
///
/// Groups up to 16 markets under a shared label, end time, and exclusivity policy.
/// Byte offsets match `docs/program/accounts.md`.
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct Event {
    pub discriminant: u8,          // offset 0    (= 3)
    pub event_id: [u8; 32],        // offset 1    — SHA-256(event label)
    pub admin: Pubkey,             // offset 33
    pub end_time: i64,             // offset 65
    pub is_exclusive: bool,        // offset 73   — if true, ResolveEvent forces non-winners to NO
    pub resolved: bool,            // offset 74
    pub market_count: u8,          // offset 75   — filled slots (max 16)
    pub markets: [[u8; 32]; 16],   // offset 76   — 16 × 32 = 512 bytes
    pub primary_category: u8,      // offset 588
    pub subcategory: u16,          // offset 589
    pub bump: u8,                  // offset 591
}

impl Event {
    pub const SIZE: usize = 592;
    pub const DISCRIMINANT: u8 = 3;
    pub const MAX_MARKETS: usize = 16;
}
