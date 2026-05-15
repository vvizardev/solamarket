/// Instruction 10 — AddMarketToEvent
///
/// Links an existing market to an event. Sets market.event = event_pubkey
/// and appends the market into event.markets[].
///
/// Accounts:
///   0  signer    admin
///   1  writable  event PDA
///   2  writable  market PDA
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::PredictionMarketError,
    state::{Event, Market, DEFAULT_PUBKEY},
    utils::pda::{verify_event_pda, verify_market_pda},
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [admin_ai, event_ai, market_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

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
    if event.market_count as usize >= Event::MAX_MARKETS {
        return Err(PredictionMarketError::EventFull.into());
    }

    let mut market: Market = {
        let d = market_ai.try_borrow_data()?;
        Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if market.discriminant != Market::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

    if market.admin != event.admin {
        return Err(PredictionMarketError::EventAdminMismatch.into());
    }
    if market.event != DEFAULT_PUBKEY {
        return Err(PredictionMarketError::MarketAlreadyInEvent.into());
    }

    // Link market → event
    market.event = *event_ai.key();

    // Append market to event.markets[]
    event.markets[event.market_count as usize] = *market_ai.key();
    event.market_count += 1;

    {
        let mut event_data = event_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *event_data, &event)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }
    {
        let mut market_data = market_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *market_data, &market)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    Ok(())
}
