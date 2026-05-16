/// Instruction 1 — UpdateGlobalConfig
///
/// Updates fee recipient and collateral mint. **Only** [`GlobalConfig::admin`] may invoke this:
/// account 0 must sign and its pubkey must match the stored admin.
///
/// Accounts:
///   0  signer           admin (must equal `GlobalConfig::admin`)
///   1  writable         global_config PDA  `[b"global_config"]`
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    error::PredictionMarketError, state::GlobalConfig, utils::pda::verify_global_config_pda,
};

#[derive(BorshDeserialize)]
pub struct UpdateGlobalConfigArgs {
    pub fee_recipient: Pubkey,
    pub collateral_mint: Pubkey,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [admin_ai, global_config_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let gc_data = global_config_ai.try_borrow_data()?;
    let mut gc =
        GlobalConfig::try_from_slice(&gc_data).map_err(|_| ProgramError::InvalidAccountData)?;
    drop(gc_data);

    if gc.discriminant != GlobalConfig::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }

    verify_global_config_pda(global_config_ai.key(), gc.bump, program_id)?;

    if admin_ai.key() != &gc.admin {
        return Err(PredictionMarketError::NotGlobalAdmin.into());
    }

    let args = UpdateGlobalConfigArgs::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    gc.fee_recipient = args.fee_recipient;
    gc.collateral_mint = args.collateral_mint;

    let mut out = global_config_ai.try_borrow_mut_data()?;
    borsh::to_writer(&mut *out, &gc).map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
