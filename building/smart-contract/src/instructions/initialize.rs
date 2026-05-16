/// Instruction 0 — Initialize
///
/// Creates the singleton [`GlobalConfig`](crate::state::GlobalConfig) PDA for program-wide settings.
///
/// Accounts:
///   0  writable signer  admin (stored in `GlobalConfig::admin`)
///   1  writable         global_config PDA  `[b"global_config"]`
///   2  —                system_program
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::Pubkey,
    seeds,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::PredictionMarketError,
    state::{GlobalConfig, DISCRIMINANT_GLOBAL_CONFIG},
    utils::pda::{find_global_config_pda, SEED_GLOBAL_CONFIG},
};

#[derive(BorshDeserialize)]
pub struct InitializeArgs {
    pub fee_recipient: Pubkey,
    pub collateral_mint: Pubkey,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [admin_ai, global_config_ai, _system_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let args =
        InitializeArgs::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;

    let (expected_key, bump) = find_global_config_pda(program_id);
    if global_config_ai.key() != &expected_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let lamports = Rent::get()?.minimum_balance(GlobalConfig::SIZE);
    let bump_arr = [bump];
    let seeds_bundle = seeds!(SEED_GLOBAL_CONFIG, &bump_arr);
    CreateAccount {
        from: admin_ai,
        to: global_config_ai,
        lamports,
        space: GlobalConfig::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[Signer::from(&seeds_bundle)])?;

    let cfg = GlobalConfig {
        discriminant: DISCRIMINANT_GLOBAL_CONFIG,
        admin: *admin_ai.key(),
        fee_recipient: args.fee_recipient,
        collateral_mint: args.collateral_mint,
        bump,
    };

    let mut cfg_data = global_config_ai.try_borrow_mut_data()?;
    borsh::to_writer(&mut *cfg_data, &cfg).map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
