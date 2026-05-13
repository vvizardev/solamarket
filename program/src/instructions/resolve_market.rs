use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::PredictionMarketError, state::Market};

/// Set `market.resolved = true` and record the winning outcome.
///
/// Accounts:
///   0. `[signer]`  admin
///   1. `[writable]` market PDA
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], outcome: u8) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin_ai  = next_account_info(iter)?;
    let market_ai = next_account_info(iter)?;

    if !admin_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let mut market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if market.admin != *admin_ai.key {
        return Err(PredictionMarketError::NotMarketAdmin.into());
    }
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }
    if outcome != 1 && outcome != 2 {
        return Err(PredictionMarketError::InvalidWinningOutcome.into());
    }

    market.resolved        = true;
    market.winning_outcome = outcome;
    market
        .serialize(&mut &mut market_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
