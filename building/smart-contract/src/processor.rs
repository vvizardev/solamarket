use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::instructions;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (&discriminant, rest) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminant {
        0  => instructions::create_market::process(program_id, accounts, rest),
        1  => instructions::split::process(program_id, accounts, rest),
        2  => instructions::merge::process(program_id, accounts, rest),
        3  => instructions::place_order::process(program_id, accounts, rest),
        4  => instructions::cancel_order::process(program_id, accounts, rest),
        5  => instructions::fill_order::process(program_id, accounts, rest),
        6  => instructions::resolve_market::process(program_id, accounts, rest),
        7  => instructions::redeem::process(program_id, accounts, rest),
        8  => instructions::tokenize_position::process(program_id, accounts, rest),
        9  => instructions::create_event::process(program_id, accounts, rest),
        10 => instructions::add_market_to_event::process(program_id, accounts, rest),
        11 => instructions::resolve_event::process(program_id, accounts, rest),
        _  => Err(ProgramError::InvalidInstructionData),
    }
}
