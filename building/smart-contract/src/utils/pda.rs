use pinocchio::pubkey::{create_program_address, find_program_address, Pubkey};

pub const SEED_MARKET:          &[u8] = b"market";
pub const SEED_VAULT_AUTHORITY: &[u8] = b"vault_authority";
pub const SEED_ORDER:           &[u8] = b"order";
pub const SEED_USER_POSITION:   &[u8] = b"user_position";
pub const SEED_EVENT:           &[u8] = b"event";
pub const SEED_YES_MINT_AUTH:   &[u8] = b"yes_mint_authority";
pub const SEED_NO_MINT_AUTH:    &[u8] = b"no_mint_authority";

// ── find (off-chain style, iterates bumps) ────────────────────────────────

pub fn find_market_pda(question_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[SEED_MARKET, question_hash], program_id)
}

pub fn find_vault_authority_pda(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[SEED_VAULT_AUTHORITY, market.as_ref()], program_id)
}

pub fn find_order_pda(
    market: &Pubkey,
    user: &Pubkey,
    nonce: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    find_program_address(
        &[SEED_ORDER, market.as_ref(), user.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
}

pub fn find_user_position_pda(
    market: &Pubkey,
    user: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    find_program_address(
        &[SEED_USER_POSITION, market.as_ref(), user.as_ref()],
        program_id,
    )
}

pub fn find_event_pda(event_id: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[SEED_EVENT, event_id], program_id)
}

pub fn find_yes_mint_authority_pda(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[SEED_YES_MINT_AUTH, market.as_ref()], program_id)
}

pub fn find_no_mint_authority_pda(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[SEED_NO_MINT_AUTH, market.as_ref()], program_id)
}

// ── verify (uses stored bump — efficient on-chain check) ──────────────────

use pinocchio::program_error::ProgramError;
use crate::error::PredictionMarketError;

pub fn verify_market_pda(
    key: &Pubkey,
    question_hash: &[u8; 32],
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let expected =
        create_program_address(&[SEED_MARKET, question_hash, &[bump]], program_id)?;
    if key != &expected {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    Ok(())
}

pub fn verify_vault_authority_pda(
    key: &Pubkey,
    market: &Pubkey,
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let expected =
        create_program_address(&[SEED_VAULT_AUTHORITY, market.as_ref(), &[bump]], program_id)?;
    if key != &expected {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    Ok(())
}

pub fn verify_order_pda(
    key: &Pubkey,
    market: &Pubkey,
    user: &Pubkey,
    nonce: u64,
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let expected = create_program_address(
        &[SEED_ORDER, market.as_ref(), user.as_ref(), &nonce.to_le_bytes(), &[bump]],
        program_id,
    )?;
    if key != &expected {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    Ok(())
}

pub fn verify_user_position_pda(
    key: &Pubkey,
    market: &Pubkey,
    user: &Pubkey,
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let expected = create_program_address(
        &[SEED_USER_POSITION, market.as_ref(), user.as_ref(), &[bump]],
        program_id,
    )?;
    if key != &expected {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    Ok(())
}

pub fn verify_event_pda(
    key: &Pubkey,
    event_id: &[u8; 32],
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let expected =
        create_program_address(&[SEED_EVENT, event_id, &[bump]], program_id)?;
    if key != &expected {
        return Err(PredictionMarketError::InvalidPda.into());
    }
    Ok(())
}
