/// Instruction 1 — Split
///
/// Deposits USDC into the vault; credits yes_balance and no_balance equally.
/// Creates the UserPosition PDA on the first call for this user × market pair.
///
/// Accounts:
///   0  writable signer  user
///   1  writable         market PDA
///   2  writable         user_position PDA  (created if first deposit)
///   3  writable         user USDC ATA
///   4  writable         market vault ATA
///   5  —                vault_authority PDA
///   6  —                token_program
///   7  —                system_program
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::Pubkey,
    seeds,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition, DISCRIMINANT_USER_POSITION},
    utils::pda::{
        find_user_position_pda, find_vault_authority_pda, verify_market_pda,
        SEED_USER_POSITION,
    },
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, user_position_ai, user_ata_ai, vault_ai, vault_authority_ai, _token_program_ai, _system_program_ai, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let amount = u64::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    if amount == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    let market: Market = {
        let d = market_ai.try_borrow_data()?;
        Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if market.discriminant != Market::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

    let now = Clock::get()?.unix_timestamp;
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }
    if now >= market.end_time {
        return Err(PredictionMarketError::MarketExpired.into());
    }

    let (vault_auth_key, _) = find_vault_authority_pda(market_ai.key(), program_id);
    if vault_authority_ai.key() != &vault_auth_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (user_pos_key, user_pos_bump) =
        find_user_position_pda(market_ai.key(), user_ai.key(), program_id);
    if user_position_ai.key() != &user_pos_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let position_exists = {
        let d = user_position_ai.try_borrow_data()?;
        !d.is_empty() && d[0] == DISCRIMINANT_USER_POSITION
    };

    if !position_exists {
        let lamports = Rent::get()?.minimum_balance(UserPosition::SIZE);
        let bump_arr = [user_pos_bump];
        let up_seeds = seeds!(SEED_USER_POSITION, market_ai.key(), user_ai.key(), &bump_arr);
        CreateAccount {
            from: user_ai,
            to: user_position_ai,
            lamports,
            space: UserPosition::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[Signer::from(&up_seeds)])?;

        let new_pos = UserPosition {
            discriminant: DISCRIMINANT_USER_POSITION,
            market: *market_ai.key(),
            user: *user_ai.key(),
            yes_balance: 0,
            no_balance: 0,
            locked_yes: 0,
            locked_no: 0,
            locked_collateral: 0,
            open_orders: [[0u8; 32]; 32],
            open_order_count: 0,
            bump: user_pos_bump,
        };
        let mut pos_data = user_position_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *pos_data, &new_pos)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // Transfer USDC: user ATA → vault
    Transfer {
        from: user_ata_ai,
        to: vault_ai,
        authority: user_ai,
        amount,
    }
    .invoke()?;

    // Credit yes_balance and no_balance
    let mut pos_data = user_position_ai.try_borrow_mut_data()?;
    let mut position = UserPosition::try_from_slice(&pos_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    position.yes_balance = position
        .yes_balance
        .checked_add(amount)
        .ok_or(PredictionMarketError::Overflow)?;
    position.no_balance = position
        .no_balance
        .checked_add(amount)
        .ok_or(PredictionMarketError::Overflow)?;

    borsh::to_writer(&mut *pos_data, &position)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
