use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition},
    utils::{
        pda::{find_user_position_pda, find_vault_authority_pda, SEED_VAULT_AUTHORITY},
        token::spl_token_mint_to,
    },
};

/// Opt-in tokenization: mint real SPL YES/NO tokens from the user's internal balance.
/// The user must have pre-created their YES/NO ATAs (or pass them here for CPI creation).
///
/// Accounts:
///   0. `[writable, signer]` user
///   1. `[writable]`         market PDA
///   2. `[writable]`         user_position PDA
///   3. `[writable]`         yes_mint (Pubkey::default until first call — created here if needed)
///   4. `[writable]`         no_mint
///   5. `[writable]`         user_yes_ata
///   6. `[writable]`         user_no_ata
///   7. `[]`                 vault_authority PDA  (mint authority)
///   8. `[]`                 token_program
///   9. `[]`                 system_program
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    if amount == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    let iter = &mut accounts.iter();
    let user_ai       = next_account_info(iter)?;
    let market_ai     = next_account_info(iter)?;
    let user_pos_ai   = next_account_info(iter)?;
    let yes_mint_ai   = next_account_info(iter)?;
    let no_mint_ai    = next_account_info(iter)?;
    let user_yes_ai   = next_account_info(iter)?;
    let user_no_ai    = next_account_info(iter)?;
    let vault_auth_ai = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    if !user_ai.is_signer {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }
    if market_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidAccountOwner.into());
    }

    let mut market = Market::try_from_slice(&market_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if market.resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved.into());
    }

    let (expected_pos_pda, _) = find_user_position_pda(market_ai.key, user_ai.key, program_id);
    if user_pos_ai.key != &expected_pos_pda || user_pos_ai.owner != program_id {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let mut pos = UserPosition::try_from_slice(&user_pos_ai.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let free_yes = pos.yes_balance.saturating_sub(pos.locked_yes);
    let free_no  = pos.no_balance.saturating_sub(pos.locked_no);
    if free_yes < amount || free_no < amount {
        return Err(PredictionMarketError::InsufficientBalance.into());
    }

    // Deduct internal balances
    pos.yes_balance = pos.yes_balance.checked_sub(amount).unwrap();
    pos.no_balance  = pos.no_balance.checked_sub(amount).unwrap();
    pos.serialize(&mut &mut user_pos_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Store mint pubkeys in Market if this is the first tokenization
    if market.yes_mint == Pubkey::default() {
        market.yes_mint = *yes_mint_ai.key;
        market.no_mint  = *no_mint_ai.key;
        market
            .serialize(&mut &mut market_ai.data.borrow_mut()[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    // Mint YES tokens to user
    let (expected_vault_auth, vault_auth_bump) =
        find_vault_authority_pda(market_ai.key, program_id);
    if vault_auth_ai.key != &expected_vault_auth {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let vault_auth_seeds: &[&[u8]] =
        &[SEED_VAULT_AUTHORITY, market_ai.key.as_ref(), &[vault_auth_bump]];

    invoke_signed(
        &spl_token_mint_to(yes_mint_ai.key, user_yes_ai.key, vault_auth_ai.key, amount)?,
        &[yes_mint_ai.clone(), user_yes_ai.clone(), vault_auth_ai.clone(), token_program.clone()],
        &[vault_auth_seeds],
    )?;

    invoke_signed(
        &spl_token_mint_to(no_mint_ai.key, user_no_ai.key, vault_auth_ai.key, amount)?,
        &[no_mint_ai.clone(), user_no_ai.clone(), vault_auth_ai.clone(), token_program.clone()],
        &[vault_auth_seeds],
    )?;

    Ok(())
}
