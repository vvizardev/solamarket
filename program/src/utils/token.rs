/// Hand-encoded SPL Token program instructions.
/// This avoids the spl-token crate dependency, which introduces conflicting
/// transitive solana-program versions in the workspace.
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// SPL Token Program.
pub const TOKEN_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// SPL Associated Token Account Program.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bRS");

/// SPL Token `Transfer` instruction.
/// Layout: [3u8 (Transfer), amount_le_u64]
pub fn spl_token_transfer(
    source:    &Pubkey,
    dest:      &Pubkey,
    authority: &Pubkey,
    amount:    u64,
) -> Result<Instruction, ProgramError> {
    let mut data = [0u8; 9];
    data[0] = 3; // Transfer discriminant
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    Ok(Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*source, false),
            AccountMeta::new(*dest, false),
            AccountMeta::new_readonly(*authority, true),
        ],
        data: data.to_vec(),
    })
}

/// SPL Token `MintTo` instruction.
/// Layout: [7u8 (MintTo), amount_le_u64]
pub fn spl_token_mint_to(
    mint:          &Pubkey,
    dest:          &Pubkey,
    mint_authority: &Pubkey,
    amount:        u64,
) -> Result<Instruction, ProgramError> {
    let mut data = [0u8; 9];
    data[0] = 7; // MintTo discriminant
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    Ok(Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*dest, false),
            AccountMeta::new_readonly(*mint_authority, true),
        ],
        data: data.to_vec(),
    })
}

/// Associated Token Account `Create` instruction.
/// ATA program determines the ATA address from (wallet, mint) — no data payload.
pub fn create_associated_token_account(
    payer:          &Pubkey,
    wallet:         &Pubkey,
    mint:           &Pubkey,
) -> Instruction {
    let ata = get_associated_token_address(wallet, mint);
    Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*wallet, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![],
    }
}

/// Derive the Associated Token Address for a given (wallet, mint) pair.
/// Seeds: `[wallet, token_program_id, mint]` with `ASSOCIATED_TOKEN_PROGRAM_ID`.
pub fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            TOKEN_PROGRAM_ID.as_ref(),
            mint.as_ref(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}
