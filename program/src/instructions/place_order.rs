use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::PredictionMarketError,
    instruction::PlaceOrderArgs,
    state::{Market, Order, UserPosition},
    utils::pda::{find_order_pda, find_user_position_pda, SEED_ORDER},
};

/// Accounts:
///   0. `[writable, signer]` user
///   1. `[writable]`         market PDA
///   2. `[writable]`         user_position PDA
///   3. `[writable]`         order PDA  (new account)
///   4. `[]`                 system_program
pub fn process(
    program_id: &Pubkey,
    accounts:   &[AccountInfo],
    args:        PlaceOrderArgs,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user_ai       = next_account_info(iter)?;
    let market_ai     = next_account_info(iter)?;
    let user_pos_ai   = next_account_info(iter)?;
    let order_ai      = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    // ── basic checks ─────────────────────────────────────────────────────
    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }
    if args.price < 1 || args.price > 9_999 {
        return Err(PredictionMarketError::InvalidOrderPrice.into());
    }
    if args.size == 0 {
        return Err(PredictionMarketError::InvalidOrderSize.into());
    }
    if args.side > 1 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }

    let market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }
    let clock = Clock::get()?;
    if clock.unix_timestamp >= market.end_time {
        return Err(PredictionMarketError::MarketExpired.into());
    }

    // ── validate PDAs ─────────────────────────────────────────────────────
    let (expected_pos_pda, _) = find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &expected_pos_pda || user_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (expected_order_pda, order_bump) =
        find_order_pda(market_ai.key, user_ai.key, args.nonce, program_id);
    if order_ai.key != &expected_order_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // ── load + validate position ─────────────────────────────────────────
    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if pos.user != *user_ai.key {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    // Lock the appropriate balance
    match args.side {
        0 => {
            // Bid: lock collateral (USDC). Free collateral = no_balance - locked_collateral
            // (Split credits both yes and no balances; bids consume the no_balance proxy).
            let free = pos.no_balance.saturating_sub(pos.locked_collateral);
            if free < args.size {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            pos.locked_collateral = pos
                .locked_collateral
                .checked_add(args.size)
                .ok_or(PredictionMarketError::Overflow)?;
        }
        1 => {
            // Ask: lock YES balance
            let free_yes = pos.yes_balance.saturating_sub(pos.locked_yes);
            if free_yes < args.size {
                return Err(PredictionMarketError::InsufficientBalance.into());
            }
            pos.locked_yes = pos
                .locked_yes
                .checked_add(args.size)
                .ok_or(PredictionMarketError::Overflow)?;
        }
        _ => unreachable!(),
    }

    // ── create Order PDA account ─────────────────────────────────────────
    let rent            = Rent::get()?;
    let order_lamports  = rent.minimum_balance(Order::LEN);
    let order_seeds: &[&[u8]] = &[
        SEED_ORDER,
        market_ai.key.as_ref(),
        user_ai.key.as_ref(),
        &args.nonce.to_le_bytes(),
        &[order_bump],
    ];

    invoke_signed(
        &system_instruction::create_account(
            user_ai.key,
            order_ai.key,
            order_lamports,
            Order::LEN as u64,
            program_id,
        ),
        &[user_ai.clone(), order_ai.clone(), system_program.clone()],
        &[order_seeds],
    )?;

    // Write Order state
    let order = Order::new(
        *market_ai.key,
        *user_ai.key,
        args.side,
        args.price,
        args.size,
        args.nonce,
        clock.unix_timestamp,
        order_bump,
    );
    order
        .serialize(&mut &mut order_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Register order in UserPosition
    pos.add_open_order(order_ai.key)?;
    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
