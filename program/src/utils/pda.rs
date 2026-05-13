use solana_program::pubkey::Pubkey;

// ── seed constants ────────────────────────────────────────────────────────────

pub const SEED_MARKET:           &[u8] = b"market";
pub const SEED_VAULT_AUTHORITY:  &[u8] = b"vault_authority";
pub const SEED_ORDER:            &[u8] = b"order";
pub const SEED_USER_POSITION:    &[u8] = b"user_position";

// ── derivation helpers ────────────────────────────────────────────────────────

/// `[b"market", question_hash]`
pub fn find_market_pda(question_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_MARKET, question_hash], program_id)
}

/// `[b"vault_authority", market_pubkey]`
pub fn find_vault_authority_pda(market_pubkey: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_VAULT_AUTHORITY, market_pubkey.as_ref()], program_id)
}

/// `[b"order", market_pubkey, user_pubkey, nonce_le_bytes]`
pub fn find_order_pda(
    market_pubkey: &Pubkey,
    user_pubkey:   &Pubkey,
    nonce:         u64,
    program_id:    &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_ORDER,
            market_pubkey.as_ref(),
            user_pubkey.as_ref(),
            &nonce.to_le_bytes(),
        ],
        program_id,
    )
}

/// `[b"user_position", market_pubkey, user_pubkey]`
pub fn find_user_position_pda(
    market_pubkey: &Pubkey,
    user_pubkey:   &Pubkey,
    program_id:    &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_USER_POSITION, market_pubkey.as_ref(), user_pubkey.as_ref()],
        program_id,
    )
}
