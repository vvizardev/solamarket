use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use super::DISCRIMINANT_MARKET;

/// Fixed-size, Borsh-serialised on-chain account.
///
/// Byte layout (for `getProgramAccounts` memcmp offsets):
///   0        discriminant  u8
///   1..32    question_hash [u8;32]
///   33..64   vault         Pubkey
///   65..96   collateral_mint Pubkey
///   97..128  yes_mint      Pubkey
///   129..160 no_mint       Pubkey
///   161..168 end_time      i64
///   169      resolved      bool
///   170      winning_outcome u8
///   171..202 admin         Pubkey
///   203..210 order_count   u64
///   211      bump          u8
///
/// Total: 212 bytes
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Market {
    pub discriminant:     u8,
    pub question_hash:    [u8; 32],
    /// USDC ATA owned by the Market PDA vault authority.
    pub vault:            Pubkey,
    /// Mock USDC mint address (devnet).
    pub collateral_mint:  Pubkey,
    /// `Pubkey::default()` until `TokenizePosition` is called.
    pub yes_mint:         Pubkey,
    pub no_mint:          Pubkey,
    pub end_time:         i64,
    pub resolved:         bool,
    /// 0 = unresolved, 1 = YES wins, 2 = NO wins.
    pub winning_outcome:  u8,
    pub admin:            Pubkey,
    pub order_count:      u64,
    pub bump:             u8,
}

impl Market {
    pub const LEN: usize = 212;

    pub fn new(
        question_hash: [u8; 32],
        vault: Pubkey,
        collateral_mint: Pubkey,
        end_time: i64,
        admin: Pubkey,
        bump: u8,
    ) -> Self {
        Self {
            discriminant:    DISCRIMINANT_MARKET,
            question_hash,
            vault,
            collateral_mint,
            yes_mint:        Pubkey::default(),
            no_mint:         Pubkey::default(),
            end_time,
            resolved:        false,
            winning_outcome: 0,
            admin,
            order_count:     0,
            bump,
        }
    }
}
