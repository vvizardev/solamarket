use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

/// On-chain Order account — 107 bytes.
///
/// Byte offsets match `docs/program/accounts.md`.
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct Order {
    pub discriminant: u8,  // offset 0  (= 1)
    pub market: Pubkey,    // offset 1
    pub user: Pubkey,      // offset 33
    pub side: u8,          // offset 65  — 0=bid, 1=ask
    pub price: u64,        // offset 66  — basis points 1–9999
    pub size: u64,         // offset 74  — original collateral units
    pub fill_amount: u64,  // offset 82  — cumulative filled
    pub nonce: u64,        // offset 90  — client-chosen PDA nonce
    pub created_at: i64,   // offset 98  — Unix timestamp
    pub bump: u8,          // offset 106
}

impl Order {
    pub const SIZE: usize = 107;
    pub const DISCRIMINANT: u8 = 1;

    pub fn remaining(&self) -> u64 {
        self.size.saturating_sub(self.fill_amount)
    }

    pub fn is_fully_filled(&self) -> bool {
        self.fill_amount >= self.size
    }
}
