use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::InstructionData;
use crate::instructions::{
    cancel_order, create_market, fill_order, merge, place_order, redeem, resolve_market, split,
    tokenize_position,
};

pub fn process_instruction(
    program_id:       &Pubkey,
    accounts:         &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = InstructionData::try_from_slice(instruction_data)?;

    match instruction {
        InstructionData::CreateMarket(args)    => create_market::process(program_id, accounts, args),
        InstructionData::Split(amount)         => split::process(program_id, accounts, amount),
        InstructionData::Merge(amount)         => merge::process(program_id, accounts, amount),
        InstructionData::PlaceOrder(args)      => place_order::process(program_id, accounts, args),
        InstructionData::CancelOrder(args)     => cancel_order::process(program_id, accounts, args),
        InstructionData::FillOrder(args)       => fill_order::process(program_id, accounts, args),
        InstructionData::ResolveMarket(outcome)=> resolve_market::process(program_id, accounts, outcome),
        InstructionData::Redeem(amount)        => redeem::process(program_id, accounts, amount),
        InstructionData::TokenizePosition(amt) => tokenize_position::process(program_id, accounts, amt),
    }
}
