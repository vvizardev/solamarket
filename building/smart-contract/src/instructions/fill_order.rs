/// Instruction 5 — FillOrder
///
/// Permissionless keeper instruction. Validates two crossing orders (bid >= ask),
/// swaps YES/NO balances, applies Polymarket-style fees (taker curve, maker rebate,
/// keeper reward, treasury), and closes fully-filled Order PDAs returning rent.
///
/// Fee math (u128 intermediates to prevent overflow):
///   fill_cost       = fill_size × bid.price / 10_000
///   curve           = bid.price × (10_000 − bid.price)
///   taker_fee       = fill_cost × curve × taker_curve_numer
///                     / (taker_curve_denom × 10_000 × 10_000)
///   maker_fee       = fill_cost × maker_fee_bps / 10_000
///   maker_rebate    = taker_fee × maker_rebate_of_taker_bps / 10_000
///   keeper_reward   = taker_fee × keeper_reward_of_taker_bps / 10_000
///   treasury_share  = taker_fee − maker_rebate − keeper_reward
///
/// Accounts:
///   0  writable signer  keeper
///   1  —                market PDA
///   2  writable         bid_order PDA
///   3  writable         ask_order PDA
///   4  writable         bid UserPosition
///   5  writable         ask UserPosition
///   6  writable         keeper UserPosition  (receives keeper_reward as no_balance)
///   7  writable         fee_recipient UserPosition  (market.fee_recipient_user × market)
use borsh::BorshDeserialize;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::PredictionMarketError,
    state::{Market, Order, UserPosition},
    utils::pda::{verify_market_pda, verify_order_pda, verify_user_position_pda},
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [keeper_ai, market_ai, bid_order_ai, ask_order_ai, bid_pos_ai, ask_pos_ai, keeper_pos_ai, fee_pos_ai, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !keeper_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let fill_size =
        u64::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    if fill_size == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    // ── load accounts ─────────────────────────────────────────────────────

    let market: Market = {
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

    let mut bid_order: Order = {
        let d = bid_order_ai.try_borrow_data()?;
        Order::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    let mut ask_order: Order = {
        let d = ask_order_ai.try_borrow_data()?;
        Order::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };

    // ── validate order sides and market match ─────────────────────────────

    if bid_order.side != 0 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }
    if ask_order.side != 1 {
        return Err(PredictionMarketError::InvalidOrderSide.into());
    }
    if &bid_order.market != market_ai.key() || &ask_order.market != market_ai.key() {
        return Err(PredictionMarketError::MarketMismatch.into());
    }
    if bid_order.price < ask_order.price {
        return Err(PredictionMarketError::NoCrossing.into());
    }
    if fill_size > bid_order.remaining() || fill_size > ask_order.remaining() {
        return Err(PredictionMarketError::OverFill.into());
    }

    // ── verify order PDAs ─────────────────────────────────────────────────

    verify_order_pda(
        bid_order_ai.key(),
        market_ai.key(),
        &bid_order.user,
        bid_order.nonce,
        bid_order.bump,
        program_id,
    )?;
    verify_order_pda(
        ask_order_ai.key(),
        market_ai.key(),
        &ask_order.user,
        ask_order.nonce,
        ask_order.bump,
        program_id,
    )?;

    // ── load user positions ───────────────────────────────────────────────

    let mut bid_pos: UserPosition = {
        let d = bid_pos_ai.try_borrow_data()?;
        UserPosition::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    let mut ask_pos: UserPosition = {
        let d = ask_pos_ai.try_borrow_data()?;
        UserPosition::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };

    verify_user_position_pda(
        bid_pos_ai.key(),
        market_ai.key(),
        &bid_order.user,
        bid_pos.bump,
        program_id,
    )?;
    verify_user_position_pda(
        ask_pos_ai.key(),
        market_ai.key(),
        &ask_order.user,
        ask_pos.bump,
        program_id,
    )?;

    // ── fee math (u128 to avoid overflow) ─────────────────────────────────

    let fill_cost: u64 =
        (fill_size as u128 * bid_order.price as u128 / 10_000) as u64;

    let taker_fee: u64 = if market.taker_curve_numer > 0 && market.taker_curve_denom > 0 {
        let curve = bid_order.price as u128 * (10_000 - bid_order.price as u128);
        ((fill_cost as u128 * curve * market.taker_curve_numer as u128)
            / (market.taker_curve_denom as u128 * 10_000u128 * 10_000u128))
            as u64
    } else {
        0
    };

    let maker_fee: u64 = (fill_cost as u128 * market.maker_fee_bps as u128 / 10_000) as u64;

    let maker_rebate: u64 =
        (taker_fee as u128 * market.maker_rebate_of_taker_bps as u128 / 10_000) as u64;
    let keeper_reward: u64 =
        (taker_fee as u128 * market.keeper_reward_of_taker_bps as u128 / 10_000) as u64;
    let treasury_share: u64 = taker_fee
        .saturating_sub(maker_rebate)
        .saturating_sub(keeper_reward);

    // ── maker / taker determination ───────────────────────────────────────
    // Older created_at = maker. Tie-break: lexicographically smaller key = maker.

    let bid_is_maker = bid_order.created_at < ask_order.created_at
        || (bid_order.created_at == ask_order.created_at
            && bid_order_ai.key() < ask_order_ai.key());

    // ── apply balance updates ─────────────────────────────────────────────

    if bid_is_maker {
        // Case B: taker = ask, maker = bid
        // Maker bid:  locked_collateral -= fill_cost − maker_rebate
        //             yes_balance += fill_size
        let bid_debit = fill_cost.saturating_sub(maker_rebate);
        bid_pos.locked_collateral =
            bid_pos.locked_collateral.saturating_sub(bid_debit);
        bid_pos.yes_balance = bid_pos
            .yes_balance
            .checked_add(fill_size)
            .ok_or(PredictionMarketError::Overflow)?;

        // Taker ask:  locked_yes -= fill_size
        //             no_balance += fill_cost − taker_fee − maker_fee
        ask_pos.locked_yes = ask_pos.locked_yes.saturating_sub(fill_size);
        let ask_proceeds = fill_cost
            .saturating_sub(taker_fee)
            .saturating_sub(maker_fee);
        ask_pos.no_balance = ask_pos
            .no_balance
            .checked_add(ask_proceeds)
            .ok_or(PredictionMarketError::Overflow)?;
    } else {
        // Case A: taker = bid, maker = ask
        // Taker bid:  locked_collateral -= fill_cost + taker_fee
        //             yes_balance += fill_size
        let bid_debit = fill_cost
            .checked_add(taker_fee)
            .ok_or(PredictionMarketError::Overflow)?;
        bid_pos.locked_collateral =
            bid_pos.locked_collateral.saturating_sub(bid_debit);
        bid_pos.yes_balance = bid_pos
            .yes_balance
            .checked_add(fill_size)
            .ok_or(PredictionMarketError::Overflow)?;

        // Maker ask:  locked_yes -= fill_size
        //             no_balance += fill_cost − maker_fee + maker_rebate
        ask_pos.locked_yes = ask_pos.locked_yes.saturating_sub(fill_size);
        let ask_proceeds = fill_cost
            .saturating_sub(maker_fee)
            .checked_add(maker_rebate)
            .ok_or(PredictionMarketError::Overflow)?;
        ask_pos.no_balance = ask_pos
            .no_balance
            .checked_add(ask_proceeds)
            .ok_or(PredictionMarketError::Overflow)?;
    }

    // ── keeper reward ─────────────────────────────────────────────────────

    let mut keeper_pos: UserPosition = {
        let d = keeper_pos_ai.try_borrow_data()?;
        UserPosition::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    keeper_pos.no_balance = keeper_pos
        .no_balance
        .checked_add(keeper_reward)
        .ok_or(PredictionMarketError::Overflow)?;

    // ── treasury / fee_recipient ──────────────────────────────────────────

    let mut fee_pos: UserPosition = {
        let d = fee_pos_ai.try_borrow_data()?;
        UserPosition::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    fee_pos.no_balance = fee_pos
        .no_balance
        .checked_add(
            treasury_share
                .checked_add(maker_fee)
                .ok_or(PredictionMarketError::Overflow)?,
        )
        .ok_or(PredictionMarketError::Overflow)?;

    // ── update fill amounts ───────────────────────────────────────────────

    bid_order.fill_amount += fill_size;
    ask_order.fill_amount += fill_size;

    // ── persist all positions ─────────────────────────────────────────────

    {
        let mut d = bid_pos_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &bid_pos).map_err(|_| ProgramError::InvalidAccountData)?;
    }
    {
        let mut d = ask_pos_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &ask_pos).map_err(|_| ProgramError::InvalidAccountData)?;
    }
    {
        let mut d = keeper_pos_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &keeper_pos)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }
    {
        let mut d = fee_pos_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &fee_pos).map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // ── persist or close bid order ────────────────────────────────────────

    if bid_order.is_fully_filled() {
        bid_pos.remove_order(bid_order_ai.key());
        {
            let mut d = bid_pos_ai.try_borrow_mut_data()?;
            borsh::to_writer(&mut *d, &bid_pos)
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        // Return rent to bid placer's position account
        let rent = bid_order_ai.lamports();
        *bid_pos_ai.try_borrow_mut_lamports()? += rent;
        bid_order_ai.close()?;
    } else {
        let mut d = bid_order_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &bid_order)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // ── persist or close ask order ────────────────────────────────────────

    if ask_order.is_fully_filled() {
        ask_pos.remove_order(ask_order_ai.key());
        {
            let mut d = ask_pos_ai.try_borrow_mut_data()?;
            borsh::to_writer(&mut *d, &ask_pos)
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        // Return rent to ask placer's position account
        let rent = ask_order_ai.lamports();
        *ask_pos_ai.try_borrow_mut_lamports()? += rent;
        ask_order_ai.close()?;
    } else {
        let mut d = ask_order_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *d, &ask_order)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    Ok(())
}
