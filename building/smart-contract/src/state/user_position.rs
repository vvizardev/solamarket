use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

/// On-chain UserPosition account — 1131 bytes.
///
/// Tracks internal YES/NO balances and locked collateral per user per market.
/// Byte offsets match `docs/program/accounts.md`.
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct UserPosition {
    pub discriminant: u8,           // offset 0    (= 2)
    pub market: Pubkey,             // offset 1
    pub user: Pubkey,               // offset 33
    pub yes_balance: u64,           // offset 65
    pub no_balance: u64,            // offset 73
    pub locked_yes: u64,            // offset 81   — YES locked in open ask orders
    pub locked_no: u64,             // offset 89   — reserved for future NO-side orders
    pub locked_collateral: u64,     // offset 97   — USDC locked in open bid orders
    pub open_orders: [[u8; 32]; 32],// offset 105  — 32 × 32 = 1024 bytes
    pub open_order_count: u8,       // offset 1129
    pub bump: u8,                   // offset 1130
}

impl UserPosition {
    pub const SIZE: usize = 1131;
    pub const DISCRIMINANT: u8 = 2;
    pub const MAX_OPEN_ORDERS: usize = 32;

    /// Add an order PDA pubkey to the open_orders array.
    pub fn add_order(&mut self, order_key: &Pubkey) -> bool {
        if self.open_order_count as usize >= Self::MAX_OPEN_ORDERS {
            return false;
        }
        self.open_orders[self.open_order_count as usize] = *order_key;
        self.open_order_count += 1;
        true
    }

    /// Remove an order PDA pubkey from the open_orders array (swap-remove).
    pub fn remove_order(&mut self, order_key: &Pubkey) {
        for i in 0..self.open_order_count as usize {
            if &self.open_orders[i] == order_key {
                let last = self.open_order_count as usize - 1;
                self.open_orders[i] = self.open_orders[last];
                self.open_orders[last] = [0u8; 32];
                self.open_order_count -= 1;
                return;
            }
        }
    }
}
