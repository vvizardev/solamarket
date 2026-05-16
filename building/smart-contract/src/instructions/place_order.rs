/// Instruction 5 — PlaceOrder
///
/// Creates an Order PDA and locks the appropriate balance:
///   - bid: locks collateral proportional to size × price / 10_000
///   - ask: locks YES tokens equal to the order size
///
/// Accounts:
///   0  writable signer  user
///   1  —                market PDA
///   2  writable         user_position PDA
///   3  writable         order PDA  (created here)
///   4  —                system_program
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::Pubkey,
    seeds,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::PredictionMarketError,
    state::{Market, Order, UserPosition, DISCRIMINANT_ORDER},
    utils::pda::{find_order_pda, verify_market_pda, verify_user_position_pda, SEED_ORDER},
};

#[derive(BorshDeserialize)]
pub struct PlaceOrderArgs {
    pub side: u8,
    pub price: u64,
    pub size: u64,
    pub nonce: u64,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, user_position_ai, order_ai, _system_program_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let args =
        PlaceOrderArgs::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;

    if args.side > 1 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }
    if args.price < 1 || args.price > 9999 {
        return Err(PredictionMarketError::InvalidOrderPrice.into());
    }
    if args.size == 0 {
        return Err(PredictionMarketError::InvalidOrderSize.into());
    }

    let market: Market = {
        let d = market_ai.try_borrow_data()?;
        Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if market.discriminant != Market::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

    let clock = Clock::get()?;
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }
    if clock.unix_timestamp >= market.end_time {
        return Err(PredictionMarketError::MarketExpired.into());
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

    if position.open_order_count as usize >= UserPosition::MAX_OPEN_ORDERS {
        return Err(PredictionMarketError::OpenOrdersFull.into());
    }

    // Lock collateral or YES tokens depending on side
    match args.side {
        0 => {
            // bid — lock collateral = size * price / 10_000
            let lock_amount = (args.size as u128 * args.price as u128 / 10_000) as u64;
            let free_collateral = position
                .yes_balance
                .checked_add(position.no_balance)
                .ok_or(PredictionMarketError::Overflow)?
                .checked_sub(
                    position
                        .locked_collateral
                        .checked_add(position.locked_yes)
                        .ok_or(PredictionMarketError::Overflow)?,
                )
                .ok_or(PredictionMarketError::InsufficientBalance)?;
            if lock_amount > free_collateral {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            position.locked_collateral = position
                .locked_collateral
                .checked_add(lock_amount)
                .ok_or(PredictionMarketError::Overflow)?;
        }
        1 => {
            // ask — lock YES tokens
            let free_yes = position
                .yes_balance
                .checked_sub(position.locked_yes)
                .ok_or(PredictionMarketError::InsufficientBalance)?;
            if args.size > free_yes {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            position.locked_yes = position
                .locked_yes
                .checked_add(args.size)
                .ok_or(PredictionMarketError::Overflow)?;
        }
        _ => unreachable!(),
    }

    // Derive Order PDA
    let nonce_bytes = args.nonce.to_le_bytes();
    let (order_key, order_bump) =
        find_order_pda(market_ai.key(), user_ai.key(), args.nonce, program_id);
    if order_ai.key() != &order_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // Create Order account
    let lamports = Rent::get()?.minimum_balance(Order::SIZE);
    let bump_arr = [order_bump];
    let order_seeds = seeds!(SEED_ORDER, market_ai.key(), user_ai.key(), &nonce_bytes, &bump_arr);
    CreateAccount {
        from: user_ai,
        to: order_ai,
        lamports,
        space: Order::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[Signer::from(&order_seeds)])?;

    let order = Order {
        discriminant: DISCRIMINANT_ORDER,
        market: *market_ai.key(),
        user: *user_ai.key(),
        side: args.side,
        price: args.price,
        size: args.size,
        fill_amount: 0,
        nonce: args.nonce,
        created_at: clock.unix_timestamp,
        bump: order_bump,
    };
    {
        let mut order_data = order_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *order_data, &order)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // Register order in UserPosition
    position.add_order(order_ai.key());
    {
        let mut pos_data = user_position_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *pos_data, &position)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    Ok(())
}
