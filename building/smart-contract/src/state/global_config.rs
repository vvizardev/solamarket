use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

/// Singleton program-global configuration PDA (`seeds`: `[b"global_config"]`).
///
/// Stores the protocol admin, fee recipient, and canonical collateral mint;
/// only `admin` may change mutable fields via [`crate::instructions::update_global_config`].
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct GlobalConfig {
    pub discriminant: u8,
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub collateral_mint: Pubkey,
    pub bump: u8,
}

impl GlobalConfig {
    /// Borsh size: `u8` + 3×`Pubkey` + `u8`
    pub const SIZE: usize = 98;
    pub const DISCRIMINANT: u8 = 4;
}
