use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::PredictionMarketError,
    instruction::CancelOrderArgs,
    state::{Order, UserPosition},
    utils::pda::{find_order_pda, find_user_position_pda},
};

/// Cancel a resting order, release locked balance, close Order PDA (lamports → user).
///
/// Accounts:
///   0. `[writable, signer]` user
///   1. `[]`                 market PDA
///   2. `[writable]`         user_position PDA
///   3. `[writable]`         order PDA
pub fn process(
    program_id: &Pubkey,
    accounts:   &[AccountInfo],
    args:        CancelOrderArgs,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user_ai      = next_account_info(iter)?;
    let market_ai    = next_account_info(iter)?;
    let user_pos_ai  = next_account_info(iter)?;
    let order_ai     = next_account_info(iter)?;

    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    // Validate PDAs
    let (expected_order_pda, _) =
        find_order_pda(market_ai.key, user_ai.key, args.nonce, program_id);
    if order_ai.key != &expected_order_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    if order_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let (expected_pos_pda, _) = find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &expected_pos_pda || user_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let order = Order::try_from_slice(&order_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if order.user != *user_ai.key {
        return Err(PredictionMarketError::NotOrderOwner.into());
    }

    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Release locked balance for unfilled portion
    let remaining = order.remaining();
    match order.side {
        0 => {
            pos.locked_collateral = pos.locked_collateral.saturating_sub(remaining);
        }
        1 => {
            pos.locked_yes = pos.locked_yes.saturating_sub(remaining);
        }
        _ => return Err(ProgramError::InvalidAccountData),
    }

    // Unregister from open_orders list
    pos.remove_open_order(order_ai.key);

    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Close Order PDA — return lamports to user
    let dest_lamports      = user_ai.lamports();
    let order_lamports     = order_ai.lamports();
    **user_ai.lamports.borrow_mut()   = dest_lamports.checked_add(order_lamports).unwrap();
    **order_ai.lamports.borrow_mut()  = 0;
    order_ai.data.borrow_mut().fill(0);

    Ok(())
}
