use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition},
    utils::{
        pda::{find_user_position_pda, find_vault_authority_pda, SEED_VAULT_AUTHORITY},
        token::spl_token_transfer,
    },
};

/// Burn equal YES + NO balances; withdraw `amount` USDC from vault back to user.
///
/// Accounts:
///   0. `[writable, signer]` user
///   1. `[writable]`         market PDA
///   2. `[writable]`         user_position PDA
///   3. `[writable]`         user_usdc_ata
///   4. `[writable]`         market vault ATA
///   5. `[]`                 vault_authority PDA
///   6. `[]`                 token_program
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    if amount == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    let iter = &mut accounts.iter();
    let user_ai        = next_account_info(iter)?;
    let market_ai      = next_account_info(iter)?;
    let user_pos_ai    = next_account_info(iter)?;
    let user_usdc_ai   = next_account_info(iter)?;
    let vault_ai       = next_account_info(iter)?;
    let vault_auth_ai  = next_account_info(iter)?;
    let token_program  = next_account_info(iter)?;

    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let _market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // validate vault_authority PDA
    let (expected_vault_auth, vault_auth_bump) =
        find_vault_authority_pda(market_ai.key, program_id);
    if vault_auth_ai.key != &expected_vault_auth {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // validate user_position PDA
    let (expected_pos_pda, _) = find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &expected_pos_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    if user_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if pos.user != *user_ai.key {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    // Check sufficient free (unlocked) balances
    let free_yes = pos.yes_balance.saturating_sub(pos.locked_yes);
    let free_no  = pos.no_balance.saturating_sub(pos.locked_no);
    if free_yes < amount || free_no < amount {
        return Err(PredictionMarketError::InsufficientBalance.into());
    }

    // Deduct balances
    pos.yes_balance = pos
        .yes_balance
        .checked_sub(amount)
        .ok_or(PredictionMarketError::Overflow)?;
    pos.no_balance = pos
        .no_balance
        .checked_sub(amount)
        .ok_or(PredictionMarketError::Overflow)?;

    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Transfer USDC vault → user (vault_authority signs)
    let vault_auth_seeds: &[&[u8]] =
        &[SEED_VAULT_AUTHORITY, market_ai.key.as_ref(), &[vault_auth_bump]];

    invoke_signed(
        &spl_token_transfer(vault_ai.key, user_usdc_ai.key, vault_auth_ai.key, amount)?,
        &[vault_ai.clone(), user_usdc_ai.clone(), vault_auth_ai.clone(), token_program.clone()],
        &[vault_auth_seeds],
    )?;

    Ok(())
}
