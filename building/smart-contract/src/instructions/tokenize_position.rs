/// Instruction 10 — TokenizePosition
///
/// Converts internal YES/NO balances to real SPL tokens (opt-in).
/// Lazily initialises YES and NO mints on first call.
///
/// Accounts:
///    0  writable signer  user
///    1  writable         market PDA
///    2  writable         user_position PDA
///    3  writable         yes_mint
///    4  writable         no_mint
///    5  writable         user YES ATA
///    6  writable         user NO ATA
///    7  —                yes_mint_authority PDA  [b"yes_mint_authority", market]
///    8  —                no_mint_authority PDA   [b"no_mint_authority", market]
///    9  —                system_program
///   10  —                token_program
///   11  —                associated_token_program
///   12  —                rent sysvar
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
use pinocchio_token::instructions::{InitializeMint2, MintTo};

use crate::{
    error::PredictionMarketError,
    state::{Market, UserPosition, DEFAULT_PUBKEY},
    utils::pda::{
        find_no_mint_authority_pda, find_yes_mint_authority_pda, verify_market_pda,
        verify_user_position_pda, SEED_NO_MINT_AUTH, SEED_YES_MINT_AUTH,
    },
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user_ai, market_ai, user_position_ai, yes_mint_ai, no_mint_ai, user_yes_ata_ai, user_no_ata_ai, yes_mint_auth_ai, no_mint_auth_ai, system_program_ai, token_program_ai, _ata_program_ai, _rent_sysvar_ai, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user_ai.is_signer() {
        return Err(PredictionMarketError::MissingRequiredSigner.into());
    }

    let amount = u64::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    if amount == 0 {
        return Err(PredictionMarketError::ZeroAmount.into());
    }

    let mut market: Market = {
        let d = market_ai.try_borrow_data()?;
        Market::try_from_slice(&d).map_err(|_| ProgramError::InvalidAccountData)?
    };
    if market.discriminant != Market::DISCRIMINANT {
        return Err(PredictionMarketError::InvalidDiscriminant.into());
    }
    verify_market_pda(market_ai.key(), &market.question_hash, market.bump, program_id)?;

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

    if position.yes_balance < amount || position.no_balance < amount {
        return Err(PredictionMarketError::InsufficientBalance.into());
    }

    let (yes_auth_key, yes_auth_bump) =
        find_yes_mint_authority_pda(market_ai.key(), program_id);
    let (no_auth_key, no_auth_bump) =
        find_no_mint_authority_pda(market_ai.key(), program_id);

    if yes_mint_auth_ai.key() != &yes_auth_key || no_mint_auth_ai.key() != &no_auth_key {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    let spl_mint_size: usize = 82;

    // Lazily initialise YES mint on first tokenization
    if market.yes_mint == DEFAULT_PUBKEY {
        let lamports = Rent::get()?.minimum_balance(spl_mint_size);
        CreateAccount {
            from: user_ai,
            to: yes_mint_ai,
            lamports,
            space: spl_mint_size as u64,
            owner: token_program_ai.key(),
        }
        .invoke()?;

        InitializeMint2 {
            mint: yes_mint_ai,
            decimals: 6,
            mint_authority: yes_mint_auth_ai.key(),
            freeze_authority: None,
        }
        .invoke()?;

        market.yes_mint = *yes_mint_ai.key();
    } else if yes_mint_ai.key() != &market.yes_mint {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // Lazily initialise NO mint on first tokenization
    if market.no_mint == DEFAULT_PUBKEY {
        let lamports = Rent::get()?.minimum_balance(spl_mint_size);
        CreateAccount {
            from: user_ai,
            to: no_mint_ai,
            lamports,
            space: spl_mint_size as u64,
            owner: token_program_ai.key(),
        }
        .invoke()?;

        InitializeMint2 {
            mint: no_mint_ai,
            decimals: 6,
            mint_authority: no_mint_auth_ai.key(),
            freeze_authority: None,
        }
        .invoke()?;

        market.no_mint = *no_mint_ai.key();
    } else if no_mint_ai.key() != &market.no_mint {
        return Err(PredictionMarketError::InvalidPda.into());
    }

    // Create user YES ATA if it doesn't exist
    if user_yes_ata_ai.data_is_empty() {
        CreateAta {
            funding_account: user_ai,
            account: user_yes_ata_ai,
            wallet: user_ai,
            mint: yes_mint_ai,
            system_program: system_program_ai,
            token_program: token_program_ai,
        }
        .invoke()?;
    }

    // Create user NO ATA if it doesn't exist
    if user_no_ata_ai.data_is_empty() {
        CreateAta {
            funding_account: user_ai,
            account: user_no_ata_ai,
            wallet: user_ai,
            mint: no_mint_ai,
            system_program: system_program_ai,
            token_program: token_program_ai,
        }
        .invoke()?;
    }

    // Mint YES tokens (signed by yes_mint_authority PDA)
    let yes_bump_arr = [yes_auth_bump];
    let yes_seeds = seeds!(SEED_YES_MINT_AUTH, market_ai.key(), &yes_bump_arr);
    MintTo {
        mint: yes_mint_ai,
        account: user_yes_ata_ai,
        mint_authority: yes_mint_auth_ai,
        amount,
    }
    .invoke_signed(&[Signer::from(&yes_seeds)])?;

    // Mint NO tokens (signed by no_mint_authority PDA)
    let no_bump_arr = [no_auth_bump];
    let no_seeds = seeds!(SEED_NO_MINT_AUTH, market_ai.key(), &no_bump_arr);
    MintTo {
        mint: no_mint_ai,
        account: user_no_ata_ai,
        mint_authority: no_mint_auth_ai,
        amount,
    }
    .invoke_signed(&[Signer::from(&no_seeds)])?;

    // Deduct internal balances
    position.yes_balance -= amount;
    position.no_balance -= amount;

    // Persist updated market (yes_mint / no_mint may have been set for the first time)
    {
        let mut market_data = market_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *market_data, &market)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }
    {
        let mut pos_data = user_position_ai.try_borrow_mut_data()?;
        borsh::to_writer(&mut *pos_data, &position)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    }

    Ok(())
}
