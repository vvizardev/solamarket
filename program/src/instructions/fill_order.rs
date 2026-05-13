use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::PredictionMarketError,
    instruction::FillOrderArgs,
    state::{Order, UserPosition},
    utils::pda::find_user_position_pda,
};

/// Fill two crossing orders (bid + ask). Called by any keeper; permissionless.
///
/// Fill fee: 5 bps of fill_size credited to the keeper's UserPosition.
/// The keeper must have a UserPosition in this market (or it must be created separately).
///
/// Accounts:
///   0. `[writable, signer]` keeper
///   1. `[]`                 market PDA
///   2. `[writable]`         bid_order PDA
///   3. `[writable]`         ask_order PDA
///   4. `[writable]`         bid_user_position PDA
///   5. `[writable]`         ask_user_position PDA
///   6. `[writable]`         keeper_user_position PDA  (receives fill fee)
pub fn process(
    program_id: &Pubkey,
    accounts:   &[AccountInfo],
    args:        FillOrderArgs,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let keeper_ai         = next_account_info(iter)?;
    let market_ai         = next_account_info(iter)?;
    let bid_order_ai      = next_account_info(iter)?;
    let ask_order_ai      = next_account_info(iter)?;
    let bid_pos_ai        = next_account_info(iter)?;
    let ask_pos_ai        = next_account_info(iter)?;
    let keeper_pos_ai       = next_account_info(iter)?;

    if !keeper_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    // ── load orders ───────────────────────────────────────────────────────
    if bid_order_ai.owner != program_id || ask_order_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let mut bid = Order::try_from_slice(&bid_order_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut ask = Order::try_from_slice(&ask_order_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // ── cross-market check ────────────────────────────────────────────────
    if bid.market != ask.market {
        return Err(PredictionMarketError::MarketMismatch.into());
    }
    if bid.market != *market_ai.key {
        return Err(PredictionMarketError::MarketMismatch.into());
    }

    // ── side checks ───────────────────────────────────────────────────────
    if bid.side != 0 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }
    if ask.side != 1 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }

    // ── crossing check ────────────────────────────────────────────────────
    if bid.price < ask.price {
        return Err(PredictionMarketError::NoCrossing.into());
    }

    // ── compute fill size ─────────────────────────────────────────────────
    let fill_size = args.fill_size.min(bid.remaining()).min(ask.remaining());
    if fill_size == 0 {
        return Err(PredictionMarketError::OverFill.into());
    }

    // Fill cost for bid = fill_size * price / 10_000  (USDC collateral spent)
    let fill_cost = fill_size
        .checked_mul(bid.price)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(PredictionMarketError::Overflow)?;

    // Fill fee to keeper = fill_size * 5 / 10_000  (5 bps)
    let fill_fee = fill_size
        .checked_mul(5)
        .and_then(|v| v.checked_div(10_000))
        .unwrap_or(0);

    // ── validate position PDAs ─────────────────────────────────────────────
    let (expected_bid_pos, _) = find_user_position_pda(market_ai.key, &bid.user, program_id);
    if bid_pos_ai.key != &expected_bid_pos || bid_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (expected_ask_pos, _) = find_user_position_pda(market_ai.key, &ask.user, program_id);
    if ask_pos_ai.key != &expected_ask_pos || ask_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (expected_keeper_pos, _) =
        find_user_position_pda(market_ai.key, keeper_ai.key, program_id);
    if keeper_pos_ai.key != &expected_keeper_pos || keeper_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // ── load positions ────────────────────────────────────────────────────
    let mut bid_pos = UserPosition::try_from_slice(&bid_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut ask_pos = UserPosition::try_from_slice(&ask_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut keeper_pos = UserPosition::try_from_slice(&keeper_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // ── apply balance changes ─────────────────────────────────────────────

    // Bid user: spent collateral, received YES tokens
    bid_pos.locked_collateral = bid_pos
        .locked_collateral
        .checked_sub(fill_cost)
        .ok_or(PredictionMarketError::InsufficientBalance)?;
    bid_pos.yes_balance = bid_pos
        .yes_balance
        .checked_add(fill_size)
        .ok_or(PredictionMarketError::Overflow)?;

    // Ask user: gave up YES tokens, received collateral
    ask_pos.locked_yes = ask_pos
        .locked_yes
        .checked_sub(fill_size)
        .ok_or(PredictionMarketError::InsufficientBalance)?;
    let ask_proceeds = fill_cost.saturating_sub(fill_fee);
    ask_pos.no_balance = ask_pos
        .no_balance
        .checked_add(ask_proceeds)
        .ok_or(PredictionMarketError::Overflow)?;

    // Keeper fee
    keeper_pos.no_balance = keeper_pos
        .no_balance
        .checked_add(fill_fee)
        .ok_or(PredictionMarketError::Overflow)?;

    // ── update order fill amounts ─────────────────────────────────────────
    bid.fill_amount = bid
        .fill_amount
        .checked_add(fill_size)
        .ok_or(PredictionMarketError::Overflow)?;
    ask.fill_amount = ask
        .fill_amount
        .checked_add(fill_size)
        .ok_or(PredictionMarketError::Overflow)?;

    // ── close fully-filled order accounts (return lamports) ───────────────
    if bid.is_fully_filled() {
        bid_pos.remove_open_order(bid_order_ai.key);
        let lamps = bid_order_ai.lamports();
        **bid_order_ai.lamports.borrow_mut() = 0;
        **keeper_pos_ai.lamports.borrow_mut() += lamps;
        bid_order_ai.data.borrow_mut().fill(0);
    } else {
        bid.serialize(&mut &mut bid_order_ai.data.borrow_mut()[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    if ask.is_fully_filled() {
        ask_pos.remove_open_order(ask_order_ai.key);
        let lamps = ask_order_ai.lamports();
        **ask_order_ai.lamports.borrow_mut() = 0;
        **keeper_pos_ai.lamports.borrow_mut() += lamps;
        ask_order_ai.data.borrow_mut().fill(0);
    } else {
        ask.serialize(&mut &mut ask_order_ai.data.borrow_mut()[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // ── write positions back ──────────────────────────────────────────────
    bid_pos.serialize(&mut &mut bid_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    ask_pos.serialize(&mut &mut ask_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    keeper_pos.serialize(&mut &mut keeper_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
