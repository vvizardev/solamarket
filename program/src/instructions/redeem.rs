use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::utils::token::spl_token_transfer;

use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition},
    utils::pda::{find_user_position_pda, find_vault_authority_pda, SEED_VAULT_AUTHORITY},
};

/// Redeem `amount` of winning tokens for USDC (1:1).
///
/// Accounts:
///   0. `[writable, signer]` user
///   1. `[]`                 market PDA
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
    let user_ai       = next_account_info(iter)?;
    let market_ai     = next_account_info(iter)?;
    let user_pos_ai   = next_account_info(iter)?;
    let user_usdc_ai  = next_account_info(iter)?;
    let vault_ai      = next_account_info(iter)?;
    let vault_auth_ai = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if !market.resolved {
        return Err(PredictionMarketError::MarketNotResolved.into());
    }

    let (expected_pos_pda, _) = find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &expected_pos_pda || user_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if pos.user != *user_ai.key {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    // Deduct winning balance
    match market.winning_outcome {
        1 => {
            // YES wins
            if pos.yes_balance < amount {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            pos.yes_balance = pos.yes_balance.checked_sub(amount).unwrap();
        }
        2 => {
            // NO wins
            if pos.no_balance < amount {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            pos.no_balance = pos.no_balance.checked_sub(amount).unwrap();
        }
        _ => return Err(PredictionMarketError::InvalidWinningOutcome.into()),
    }

    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Transfer USDC vault → user
    let (expected_vault_auth, vault_auth_bump) =
        find_vault_authority_pda(market_ai.key, program_id);
    if vault_auth_ai.key != &expected_vault_auth {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let vault_auth_seeds: &[&[u8]] =
        &[SEED_VAULT_AUTHORITY, market_ai.key.as_ref(), &[vault_auth_bump]];

    invoke_signed(
        &spl_token_transfer(vault_ai.key, user_usdc_ai.key, vault_auth_ai.key, amount)?,
        &[vault_ai.clone(), user_usdc_ai.clone(), vault_auth_ai.clone(), token_program.clone()],
        &[vault_auth_seeds],
    )?;

    Ok(())
}
