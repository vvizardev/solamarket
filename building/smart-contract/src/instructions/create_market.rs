/// Instruction 0 — CreateMarket
///
/// Creates the Market PDA and the USDC vault ATA owned by the vault_authority PDA.
///
/// Accounts:
///   0  writable signer  admin
///   1  writable         market PDA   [b"market", question_hash]
///   2  writable         vault ATA    (associated token account for vault_authority × collateral_mint)
///   3  —                vault_authority PDA  [b"vault_authority", market]
///   4  —                collateral_mint
///   5  —                system_program
///   6  —                token_program
///   7  —                associated_token_program
///   8  —                rent sysvar
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
use pinocchio_associated_token_account::instructions::Create as CreateAta;
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::PredictionMarketError,
    state::{Market, DEFAULT_PUBKEY, DISCRIMINANT_MARKET},
    utils::pda::{
        find_market_pda, find_vault_authority_pda, SEED_MARKET,
    },
};

#[derive(BorshDeserialize)]
pub struct CreateMarketArgs {
    pub question_hash: [u8; 32],
    pub end_time: i64,
    pub fee_recipient_user: Pubkey,
    pub taker_curve_numer: u32,
    pub taker_curve_denom: u32,
    pub maker_fee_bps: u16,
    pub maker_rebate_of_taker_bps: u16,
    pub keeper_reward_of_taker_bps: u16,
    pub primary_category: u8,
    pub subcategory: u16,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [admin_ai, market_ai, vault_ai, vault_authority_ai, collateral_mint_ai, system_program_ai, token_program_ai, _ata_program_ai, _rent_sysvar_ai, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !admin_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let args = CreateMarketArgs::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    if args.maker_rebate_of_taker_bps
        .checked_add(args.keeper_reward_of_taker_bps)
        .ok_or(PredictionMarketError::Overflow)?
        > 10_000
    {
        return Err(PredictionMarketError::InvalidOrderSize.into());
    }

    let (market_key, market_bump) = find_market_pda(&args.question_hash, program_id);
    if market_ai.key() != &market_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (vault_auth_key, _vault_auth_bump) = find_vault_authority_pda(&market_key, program_id);
    if vault_authority_ai.key() != &vault_auth_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let lamports = Rent::get()?.minimum_balance(Market::SIZE);

    // Create Market PDA account (signed by the market PDA)
    let market_bump_arr = [market_bump];
    let market_seeds = seeds!(SEED_MARKET, &args.question_hash, &market_bump_arr);
    CreateAccount {
        from: admin_ai,
        to: market_ai,
        lamports,
        space: Market::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[Signer::from(&market_seeds)])?;

    // Create vault ATA (admin pays; ATA owned by vault_authority PDA)
    CreateAta {
        funding_account: admin_ai,
        account: vault_ai,
        wallet: vault_authority_ai,
        mint: collateral_mint_ai,
        system_program: system_program_ai,
        token_program: token_program_ai,
    }
    .invoke()?;

    let market = Market {
        discriminant: DISCRIMINANT_MARKET,
        question_hash: args.question_hash,
        vault: *vault_ai.key(),
        collateral_mint: *collateral_mint_ai.key(),
        yes_mint: DEFAULT_PUBKEY,
        no_mint: DEFAULT_PUBKEY,
        end_time: args.end_time,
        resolved: false,
        winning_outcome: 0,
        admin: *admin_ai.key(),
        order_count: 0,
        event: DEFAULT_PUBKEY,
        taker_curve_numer: args.taker_curve_numer,
        taker_curve_denom: args.taker_curve_denom,
        maker_fee_bps: args.maker_fee_bps,
        maker_rebate_of_taker_bps: args.maker_rebate_of_taker_bps,
        keeper_reward_of_taker_bps: args.keeper_reward_of_taker_bps,
        fee_padding: 0,
        fee_recipient_user: args.fee_recipient_user,
        primary_category: args.primary_category,
        subcategory: args.subcategory,
        bump: market_bump,
    };

    let mut market_data = market_ai.try_borrow_mut_data()?;
    borsh::to_writer(&mut *market_data, &market)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
