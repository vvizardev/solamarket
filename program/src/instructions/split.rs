use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition},
    utils::{
        pda::{find_user_position_pda, find_vault_authority_pda},
        token::spl_token_transfer,
    },
};

/// Deposit `amount` USDC into the vault; credit `yes_balance` and `no_balance` equally.
///
/// Accounts:
///   0. `[writable, signer]` user
///   1. `[writable]`         market PDA
///   2. `[writable]`         user_position PDA  (created here if first deposit)
///   3. `[writable]`         user_usdc_ata
///   4. `[writable]`         market vault ATA
///   5. `[]`                 vault_authority PDA
///   6. `[]`                 token_program
///   7. `[]`                 system_program
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    if amount == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    let iter = &mut accounts.iter();
    let user_ai          = next_account_info(iter)?;
    let market_ai        = next_account_info(iter)?;
    let user_pos_ai      = next_account_info(iter)?;
    let user_usdc_ai     = next_account_info(iter)?;
    let vault_ai         = next_account_info(iter)?;
    let vault_auth_ai    = next_account_info(iter)?;
    let token_program    = next_account_info(iter)?;
    let system_program   = next_account_info(iter)?;

    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }

    // ── ensure vault_authority PDA is correct ────────────────────────────
    let (expected_vault_auth, _) = find_vault_authority_pda(market_ai.key, program_id);
    if vault_auth_ai.key != &expected_vault_auth {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // ── transfer USDC from user → vault ─────────────────────────────────
    invoke(
        &spl_token_transfer(user_usdc_ai.key, vault_ai.key, user_ai.key, amount)?,
        &[user_usdc_ai.clone(), vault_ai.clone(), user_ai.clone(), token_program.clone()],
    )?;

    // ── create UserPosition PDA if it doesn't exist yet ──────────────────
    let (user_pos_pda, user_pos_bump) =
        find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &user_pos_pda {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    if user_pos_ai.data_is_empty() {
        let rent     = Rent::get()?;
        let lamports = rent.minimum_balance(UserPosition::LEN);
        let pos_seeds: &[&[u8]] = &[
            crate::utils::pda::SEED_USER_POSITION,
            market_ai.key.as_ref(),
            user_ai.key.as_ref(),
            &[user_pos_bump],
        ];
        solana_program::program::invoke_signed(
            &system_instruction::create_account(
                user_ai.key,
                &user_pos_pda,
                lamports,
                UserPosition::LEN as u64,
                program_id,
            ),
            &[user_ai.clone(), user_pos_ai.clone(), system_program.clone()],
            &[pos_seeds],
        )?;
        let pos = UserPosition::new(*market_ai.key, *user_ai.key, user_pos_bump);
        pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // ── update balances ──────────────────────────────────────────────────
    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if pos.user != *user_ai.key {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    pos.yes_balance = pos
        .yes_balance
        .checked_add(amount)
        .ok_or(PredictionMarketError::Overflow)?;
    pos.no_balance = pos
        .no_balance
        .checked_add(amount)
        .ok_or(PredictionMarketError::Overflow)?;

    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
