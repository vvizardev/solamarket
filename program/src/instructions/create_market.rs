use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
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
    instruction::CreateMarketArgs,
    state::Market,
    utils::{
        pda::{find_market_pda, find_vault_authority_pda},
        token::create_associated_token_account,
    },
};

/// Accounts:
///   0. `[writable, signer]` admin
///   1. `[writable]`         market PDA
///   2. `[writable]`         vault ATA  (USDC ATA owned by vault_authority PDA)
///   3. `[]`                 vault_authority PDA
///   4. `[]`                 collateral_mint (mock USDC)
///   5. `[]`                 system_program
///   6. `[]`                 token_program
///   7. `[]`                 associated_token_program
///   8. `[]`                 rent sysvar
pub fn process(
    program_id: &Pubkey,
    accounts:   &[AccountInfo],
    args:        CreateMarketArgs,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin_ai          = next_account_info(iter)?;
    let market_ai         = next_account_info(iter)?;
    let vault_ai          = next_account_info(iter)?;
    let vault_auth_ai     = next_account_info(iter)?;
    let collateral_mint   = next_account_info(iter)?;
    let system_program    = next_account_info(iter)?;
    let token_program     = next_account_info(iter)?;
    let ata_program       = next_account_info(iter)?;
    let _rent_ai          = next_account_info(iter)?;

    // ── signer check ────────────────────────────────────────────────────
    if !admin_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    // ── PDA derivation ───────────────────────────────────────────────────
    let (market_pda, market_bump) = find_market_pda(&args.question_hash, program_id);
    if market_ai.key != &market_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let (vault_auth_pda, _vault_bump) = find_vault_authority_pda(&market_pda, program_id);
    if vault_auth_ai.key != &vault_auth_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // ── create Market PDA account ────────────────────────────────────────
    let rent          = Rent::get()?;
    let market_lamports = rent.minimum_balance(Market::LEN);
    let market_seeds  = &[
        crate::utils::pda::SEED_MARKET,
        args.question_hash.as_ref(),
        &[market_bump],
    ];

    invoke_signed(
        &system_instruction::create_account(
            admin_ai.key,
            &market_pda,
            market_lamports,
            Market::LEN as u64,
            program_id,
        ),
        &[admin_ai.clone(), market_ai.clone(), system_program.clone()],
        &[market_seeds],
    )?;

    // ── create USDC vault ATA (admin pays) ───────────────────────────────
    invoke_signed(
        &create_associated_token_account(admin_ai.key, vault_auth_ai.key, collateral_mint.key),
        &[
            admin_ai.clone(),
            vault_ai.clone(),
            vault_auth_ai.clone(),
            collateral_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            ata_program.clone(),
        ],
        &[],
    )?;

    // ── write Market state ───────────────────────────────────────────────
    let market = Market::new(
        args.question_hash,
        *vault_ai.key,
        *collateral_mint.key,
        args.end_time,
        *admin_ai.key,
        market_bump,
    );
    market
        .serialize(&mut &mut market_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
