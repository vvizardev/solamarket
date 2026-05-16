/// Instruction 8 — ResolveMarket
///
/// Admin sets the market as resolved with a YES or NO outcome.
///
/// Accounts:
///   0  signer    admin  (must equal market.admin)
///   1  writable  market PDA
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::PredictionMarketError,
    state::Market,
    utils::pda::verify_market_pda,
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [admin_ai, market_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let outcome = *data.first().ok_or(ProgramError::InvalidInstructionData)?;
    if outcome != 1 && outcome != 2 {
        return Err(PredictionMarketError::InvalidWinningOutcome.into());
    }

    let mut market: Market = {
        let d = market_ai.try_borrow_data()?;
        Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };

    if market.discriminant != Market::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

    if admin_ai.key() != &market.admin {
        return Err(PredictionMarketError::NotMarketAdmin.into());
    }
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }

    market.resolved = true;
    market.winning_outcome = outcome;

    let mut market_data = market_ai.try_borrow_mut_data()?;
    borsh::to_writer(&mut *market_data, &market)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
