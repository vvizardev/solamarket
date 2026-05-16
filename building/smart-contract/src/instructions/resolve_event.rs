/// Instruction 13 — ResolveEvent
///
/// Resolves an exclusive multi-market event atomically:
///   - Markets at winning_index → YES
///   - All other markets → NO
///   - Sets event.resolved = true
///
/// Only works on exclusive events (is_exclusive = true).
/// Non-exclusive events must resolve each market individually via ResolveMarket.
///
/// Accounts:
///   0       signer    admin
///   1       writable  event PDA
///   2..N    writable  all event.market_count market PDAs in event.markets[] order
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::PredictionMarketError,
    state::{Event, Market},
    utils::pda::{verify_event_pda, verify_market_pda},
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let admin_ai = &accounts[0];
    let event_ai = &accounts[1];
    let market_accounts = &accounts[2..];

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let winning_index =
        *data.first().ok_or(ProgramError::InvalidInstructionData)?;

    let mut event: Event = {
        let d = event_ai.try_borrow_data()?;
        Event::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if event.discriminant != Event::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_event_pda(event_ai.key(), &event.event_id, event.bump, program_id)?;

    if admin_ai.key() != &event.admin {
        return Err(PredictionMarketError::NotEventAdmin.into());
    }
    if event.resolved {
        return Err(PredictionMarketError::EventAlreadyResolved.into());
    }
    if !event.is_exclusive {
        return Err(PredictionMarketError::NotExclusiveEvent.into());
    }
    if winning_index >= event.market_count {
        return Err(PredictionMarketError::InvalidMarketIndex.into());
    }
    if market_accounts.len() != event.market_count as usize {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Resolve each market
    for (i, market_ai) in market_accounts.iter().enumerate() {
        // Verify the provided account matches the expected slot in event.markets[]
        if market_ai.key() != &event.markets[i] {
            return Err(PredictionMarketError::EventMarketMismatch.into());
        }

        let mut market: Market = {
            let d = market_ai.try_borrow_data()?;
            Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
        };
        if market.discriminant != Market::DISCRIMINANT {
            return Err(PredictionMarketError::InvalidDiscriminant.into());
        }
        verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

        if market.resolved {
            return Err(PredictionMarketError::MarketAlreadyResolved.into());
        }

        market.resolved = true;
        market.winning_outcome = if i == winning_index as usize { 1 } else { 2 };

        let mut market_data = market_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *market_data, &market)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    event.resolved = true;

    {
        let mut event_data = event_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *event_data, &event)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    Ok(())
}
