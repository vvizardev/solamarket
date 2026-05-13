use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use super::DISCRIMINANT_USER_POSITION;
use crate::error::PredictionMarketError;

/// One account per (user, market) pair.
///
/// All balance fields are in collateral units (6-decimal USDC).
///
/// Byte layout (abbreviated):
///   0          discriminant        u8
///   1..32      market              Pubkey
///   33..64     user                Pubkey
///   65..72     yes_balance         u64
///   73..80     no_balance          u64
///   81..88     locked_yes          u64  YES locked in open ask orders
///   89..96     locked_no           u64  NO  locked in open no-side orders
///   97..104    locked_collateral   u64  USDC locked in open bid orders
///   105..1128  open_orders         [Pubkey; 32]  (32 × 32 bytes = 1024)
///   1129       open_order_count    u8
///   1130       bump                u8
///
/// Total: 1 131 bytes
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserPosition {
    pub discriminant:       u8,
    pub market:             Pubkey,
    pub user:               Pubkey,
    pub yes_balance:        u64,
    pub no_balance:         u64,
    /// YES locked in open ask orders.
    pub locked_yes:         u64,
    /// NO locked in open no-bid orders.
    pub locked_no:          u64,
    /// USDC locked in open bid orders.
    pub locked_collateral:  u64,
    pub open_orders:        [Pubkey; 32],
    pub open_order_count:   u8,
    pub bump:               u8,
}

impl UserPosition {
    pub const LEN: usize = 1131;
    pub const MAX_OPEN_ORDERS: usize = 32;

    pub fn new(market: Pubkey, user: Pubkey, bump: u8) -> Self {
        Self {
            discriminant:      DISCRIMINANT_USER_POSITION,
            market,
            user,
            yes_balance:       0,
            no_balance:        0,
            locked_yes:        0,
            locked_no:         0,
            locked_collateral: 0,
            open_orders:       [Pubkey::default(); 32],
            open_order_count:  0,
            bump,
        }
    }

    /// Append an order pubkey to `open_orders`. Errors if list is full.
    pub fn add_open_order(
        &mut self,
        order_pubkey: &Pubkey,
    ) -> Result<(), PredictionMarketError> {
        if self.open_order_count as usize >= Self::MAX_OPEN_ORDERS {
            return Err(PredictionMarketError::OpenOrdersFull);
        }
        self.open_orders[self.open_order_count as usize] = *order_pubkey;
        self.open_order_count += 1;
        Ok(())
    }

    /// Remove an order pubkey from `open_orders` (swap-and-pop).
    pub fn remove_open_order(&mut self, order_pubkey: &Pubkey) {
        let count = self.open_order_count as usize;
        for i in 0..count {
            if &self.open_orders[i] == order_pubkey {
                self.open_orders[i] = self.open_orders[count - 1];
                self.open_orders[count - 1] = Pubkey::default();
                self.open_order_count -= 1;
                return;
            }
        }
    }
}
