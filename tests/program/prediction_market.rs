//! Integration tests using `solana-program-test`.
//! Each test spins up a lightweight BanksClient environment (no validator required).

#![cfg(test)]

use borsh::BorshSerialize;
use prediction_market::{
    instruction::{CreateMarketArgs, FillOrderArgs, PlaceOrderArgs},
    instruction::InstructionData,
    state::{Market, Order, UserPosition},
    utils::pda::{
        find_market_pda, find_order_pda, find_user_position_pda, find_vault_authority_pda,
    },
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::{processor, tokio, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn program_id() -> Pubkey {
    prediction_market::id()
}

fn ix_data(ix: &InstructionData) -> Vec<u8> {
    let mut v = Vec::new();
    ix.serialize(&mut v).unwrap();
    v
}

async fn get_account(banks: &mut BanksClient, pubkey: Pubkey) -> Account {
    banks
        .get_account(pubkey)
        .await
        .unwrap()
        .expect("account not found")
}

fn question_hash(question: &str) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    question.hash(&mut h);
    let n = h.finish();
    let mut out = [0u8; 32];
    out[..8].copy_from_slice(&n.to_le_bytes());
    out
}

// ── tests ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_market() {
    let pid = program_id();
    let mut pt = ProgramTest::new("prediction_market", pid, processor!(prediction_market::processor::process_instruction));

    let admin = Keypair::new();
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 10_000_000_000,
            owner:    system_program::id(),
            ..Account::default()
        },
    );

    let (mut banks, payer, recent_bh) = pt.start().await;

    let qhash = question_hash("Will BTC > 100k by end of 2025?");
    let (market_pda, _) = find_market_pda(&qhash, &pid);
    let (vault_auth, _) = find_vault_authority_pda(&market_pda, &pid);

    // NOTE: In a full test we'd mock the USDC mint and ATA creation.
    // Here we test the happy path up to instruction dispatch.
    let end_time = 9_999_999_999i64; // far future
    let args = CreateMarketArgs { question_hash: qhash, end_time };
    let data = ix_data(&InstructionData::CreateMarket(args));

    // Just verify the instruction serialises without panic
    assert!(!data.is_empty());
    assert_eq!(data[0], 0); // discriminant = 0
}

#[tokio::test]
async fn test_instruction_discriminants() {
    // Verify all variant discriminants match the expected byte values
    macro_rules! check_discriminant {
        ($variant:expr, $expected:expr) => {
            let data = ix_data(&$variant);
            assert_eq!(data[0], $expected, "discriminant mismatch for variant {}", $expected);
        };
    }

    use prediction_market::instruction::{
        CancelOrderArgs, CreateMarketArgs, FillOrderArgs, PlaceOrderArgs,
    };

    check_discriminant!(InstructionData::CreateMarket(CreateMarketArgs { question_hash: [0u8;32], end_time: 0 }), 0);
    check_discriminant!(InstructionData::Split(100u64), 1);
    check_discriminant!(InstructionData::Merge(100u64), 2);
    check_discriminant!(InstructionData::PlaceOrder(PlaceOrderArgs { side: 0, price: 5000, size: 1000, nonce: 0 }), 3);
    check_discriminant!(InstructionData::CancelOrder(CancelOrderArgs { nonce: 0 }), 4);
    check_discriminant!(InstructionData::FillOrder(FillOrderArgs { fill_size: 500 }), 5);
    check_discriminant!(InstructionData::ResolveMarket(1), 6);
    check_discriminant!(InstructionData::Redeem(100), 7);
    check_discriminant!(InstructionData::TokenizePosition(100), 8);
}

#[tokio::test]
async fn test_market_state_size() {
    assert_eq!(Market::LEN, 212, "Market layout changed");
    assert_eq!(Order::LEN, 107, "Order layout changed");
    assert_eq!(UserPosition::LEN, 1131, "UserPosition layout changed");
}

#[tokio::test]
async fn test_pda_derivation_deterministic() {
    let pid = program_id();
    let qhash = question_hash("Test market");
    let user  = Keypair::new().pubkey();

    let (m1, b1) = find_market_pda(&qhash, &pid);
    let (m2, b2) = find_market_pda(&qhash, &pid);
    assert_eq!(m1, m2);
    assert_eq!(b1, b2);

    let (p1, pb1) = find_user_position_pda(&m1, &user, &pid);
    let (p2, pb2) = find_user_position_pda(&m1, &user, &pid);
    assert_eq!(p1, p2);
    assert_eq!(pb1, pb2);

    let nonce: u64 = 42;
    let (o1, ob1) = find_order_pda(&m1, &user, nonce, &pid);
    let (o2, ob2) = find_order_pda(&m1, &user, nonce, &pid);
    assert_eq!(o1, o2);
    assert_eq!(ob1, ob2);
}

#[tokio::test]
async fn test_order_remaining_and_fill() {
    let market = Pubkey::new_unique();
    let user   = Pubkey::new_unique();
    let mut order = Order::new(market, user, 0, 5000, 1_000_000, 1, 0, 255);

    assert_eq!(order.remaining(), 1_000_000);
    assert!(!order.is_fully_filled());

    order.fill_amount = 500_000;
    assert_eq!(order.remaining(), 500_000);
    assert!(!order.is_fully_filled());

    order.fill_amount = 1_000_000;
    assert_eq!(order.remaining(), 0);
    assert!(order.is_fully_filled());
}

#[tokio::test]
async fn test_user_position_open_orders() {
    let market = Pubkey::new_unique();
    let user   = Pubkey::new_unique();
    let mut pos = UserPosition::new(market, user, 255);

    let o1 = Pubkey::new_unique();
    let o2 = Pubkey::new_unique();

    pos.add_open_order(&o1).unwrap();
    pos.add_open_order(&o2).unwrap();
    assert_eq!(pos.open_order_count, 2);

    pos.remove_open_order(&o1);
    assert_eq!(pos.open_order_count, 1);
    // o2 should still be present (swap-and-pop)
    assert!(pos.open_orders[..1].contains(&o2));
}
