/// Instruction 7 — Redeem
///
/// Post-resolution: exchange winning-outcome internal balance for USDC at 1:1.
///
/// Accounts:
///   0  writable signer  user
///   1  —                market PDA
///   2  writable         user_position PDA
///   3  writable         user USDC ATA
///   4  writable         market vault ATA
///   5  —                vault_authority PDA
///   6  —                token_program
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::Pubkey,
    seeds,
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition},
    utils::pda::{
        find_vault_authority_pda, verify_market_pda, verify_user_position_pda,
        SEED_VAULT_AUTHORITY,
    },
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, user_position_ai, user_ata_ai, vault_ai, vault_authority_ai, _token_program_ai, ..] =
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

    if !market.resolved {
        return Err(PredictionMarketError::MarketNotResolved.into());
    }

    let mut position: UserPosition = {
        let d = user_position_ai.try_borrow_data()?;
        UserPosition::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if position.discriminant != UserPosition::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_user_position_pda(
        user_position_ai.key(),
        market_ai.key(),
        user_ai.key(),
        position.bump,
        program_id,
    )?;

    match market.winning_outcome {
        1 => {
            if position.yes_balance < amount {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            position.yes_balance -= amount;
        }
        2 => {
            if position.no_balance < amount {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            position.no_balance -= amount;
        }
        _ => return Err(PredictionMarketError::InvalidWinningOutcome.into()),
    }

    {
        let mut pos_data = user_position_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *pos_data, &position)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    let (_, vault_auth_bump) = find_vault_authority_pda(market_ai.key(), program_id);
    let bump_arr = [vault_auth_bump];
    let vault_seeds = seeds!(SEED_VAULT_AUTHORITY, market_ai.key(), &bump_arr);
    Transfer {
        from: vault_ai,
        to: user_ata_ai,
        authority: vault_authority_ai,
        amount,
    }
    .invoke_signed(&[Signer::from(&vault_seeds)])?;

    Ok(())
}
