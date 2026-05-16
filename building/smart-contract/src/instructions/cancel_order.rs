/// Instruction 6 — CancelOrder
///
/// Closes the Order PDA and releases locked balance back to UserPosition.
/// Rent lamports are returned to the order placer (user).
///
/// Accounts:
///   0  writable signer  user  (must be order.user)
///   1  —                market PDA
///   2  writable         user_position PDA
///   3  writable         order PDA
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::PredictionMarketError,
    state::{Order, UserPosition},
    utils::pda::{verify_order_pda, verify_user_position_pda},
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, user_position_ai, order_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let nonce = u64::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;

    let order: Order = {
        let d = order_ai.try_borrow_data()?;
        Order::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if order.discriminant != Order::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    if &order.user != user_ai.key() {
        return Err(PredictionMarketError::NotOrderOwner.into());
    }
    verify_order_pda(
        order_ai.key(),
        market_ai.key(),
        user_ai.key(),
        nonce,
        order.bump,
        program_id,
    )?;

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

    // Release locks: remaining (unfilled) portion of the order
    let remaining = order.remaining();
    match order.side {
        0 => {
            // bid — release locked_collateral
            let fill_cost =
                (remaining as u128 * order.price as u128 / 10_000) as u64;
            position.locked_collateral =
                position.locked_collateral.saturating_sub(fill_cost);
        }
        1 => {
            // ask — release locked_yes
            position.locked_yes = position.locked_yes.saturating_sub(remaining);
        }
        _ => {}
    }

    // Remove from open_orders array
    position.remove_order(order_ai.key());

    {
        let mut pos_data = user_position_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *pos_data, &position)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // Close the Order PDA: transfer rent to user, then close the account
    {
        let rent_lamports = order_ai.lamports();
        {
            let mut user_lamps = user_ai.try_borrow_mut_lamports()?;
            *user_lamps += rent_lamports;
        }
        // close() zeros lamports, data_len, and owner
        order_ai.close()?;
    }

    Ok(())
}
