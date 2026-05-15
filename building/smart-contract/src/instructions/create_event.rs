/// Instruction 9 — CreateEvent
///
/// Creates an Event PDA that groups multiple markets under a shared label,
/// end time, and exclusivity mode.
///
/// Accounts:
///   0  writable signer  admin
///   1  writable         event PDA  [b"event", event_id]
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
    state::{Event, DISCRIMINANT_EVENT},
    utils::pda::{find_event_pda, SEED_EVENT},
};

#[derive(BorshDeserialize)]
pub struct CreateEventArgs {
    pub event_id: [u8; 32],
    pub end_time: i64,
    pub is_exclusive: bool,
    pub primary_category: u8,
    pub subcategory: u16,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [admin_ai, event_ai, _system_program_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let args =
        CreateEventArgs::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;

    let (event_key, event_bump) = find_event_pda(&args.event_id, program_id);
    if event_ai.key() != &event_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let lamports = Rent::get()?.minimum_balance(Event::SIZE);
    let bump_arr = [event_bump];
    let event_seeds = seeds!(SEED_EVENT, &args.event_id, &bump_arr);
    CreateAccount {
        from: admin_ai,
        to: event_ai,
        lamports,
        space: Event::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[Signer::from(&event_seeds)])?;

    let event = Event {
        discriminant: DISCRIMINANT_EVENT,
        event_id: args.event_id,
        admin: *admin_ai.key(),
        end_time: args.end_time,
        is_exclusive: args.is_exclusive,
        resolved: false,
        market_count: 0,
        markets: [[0u8; 32]; 16],
        primary_category: args.primary_category,
        subcategory: args.subcategory,
        bump: event_bump,
    };

    let mut event_data = event_ai.try_borrow_mut_data()?;
    borsh::to_writer(&mut *event_data, &event)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
