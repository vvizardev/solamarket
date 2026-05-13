use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use super::DISCRIMINANT_ORDER;

/// One resting limit order. Stored as a PDA; closed when fully filled or cancelled.
///
/// Byte layout:
///   0        discriminant  u8
///   1..32    market        Pubkey
///   33..64   user          Pubkey
///   65       side          u8   (0=bid, 1=ask)
///   66..73   price         u64  (basis points 0–10 000)
///   74..81   size          u64  (total collateral)
///   82..89   fill_amount   u64  (how much has been filled)
///   90..97   nonce         u64
///   98..105  created_at    i64
///   106      bump          u8
///
/// Total: 107 bytes
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Order {
    pub discriminant: u8,
    pub market:       Pubkey,
    pub user:         Pubkey,
    /// 0 = bid (buy YES), 1 = ask (sell YES).
    pub side:         u8,
    /// Price in basis points, range 1–9 999.
    pub price:        u64,
    /// Total size in collateral units (6-decimal USDC).
    pub size:         u64,
    /// Amount already filled (in collateral units).
    pub fill_amount:  u64,
    pub nonce:        u64,
    pub created_at:   i64,
    pub bump:         u8,
}

impl Order {
    pub const LEN: usize = 107;

    pub fn new(
        market: Pubkey,
        user: Pubkey,
        side: u8,
        price: u64,
        size: u64,
        nonce: u64,
        created_at: i64,
        bump: u8,
    ) -> Self {
        Self {
            discriminant: DISCRIMINANT_ORDER,
            market,
            user,
            side,
            price,
            size,
            fill_amount: 0,
            nonce,
            created_at,
            bump,
        }
    }

    #[inline]
    pub fn remaining(&self) -> u64 {
        self.size.saturating_sub(self.fill_amount)
    }

    #[inline]
    pub fn is_fully_filled(&self) -> bool {
        self.fill_amount >= self.size
    }
}
