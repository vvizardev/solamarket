use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

/// On-chain Market account — 295 bytes.
///
/// Byte offsets match the canonical layout documented in `docs/program/accounts.md`.
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct Market {
    pub discriminant: u8,          // offset 0
    pub question_hash: [u8; 32],   // offset 1
    pub vault: Pubkey,             // offset 33   — vault ATA pubkey
    pub collateral_mint: Pubkey,   // offset 65
    pub yes_mint: Pubkey,          // offset 97   — default until TokenizePosition
    pub no_mint: Pubkey,           // offset 129  — default until TokenizePosition
    pub end_time: i64,             // offset 161
    pub resolved: bool,            // offset 169
    pub winning_outcome: u8,       // offset 170  — 0=unresolved, 1=YES, 2=NO
    pub admin: Pubkey,             // offset 171
    pub order_count: u64,          // offset 203
    pub event: Pubkey,             // offset 211  — default = standalone market

    // Fee schedule
    pub taker_curve_numer: u32,          // offset 243
    pub taker_curve_denom: u32,          // offset 247
    pub maker_fee_bps: u16,              // offset 251
    pub maker_rebate_of_taker_bps: u16,  // offset 253
    pub keeper_reward_of_taker_bps: u16, // offset 255
    pub fee_padding: u16,                // offset 257  — reserved / alignment
    pub fee_recipient_user: Pubkey,      // offset 259  — treasury owner

    pub primary_category: u8,  // offset 291
    pub subcategory: u16,      // offset 292
    pub bump: u8,              // offset 294
}

impl Market {
    pub const SIZE: usize = 295;
    pub const DISCRIMINANT: u8 = 0;
}
