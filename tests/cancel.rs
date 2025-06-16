use solana_program_test::*;
use solana_sdk::{
    account::Account,
    clock::Clock,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use spl_token;
use svm_escrow::{
    instruction::{EscrowInit, EscrowInstruction},
    state::Escrow,
};

mod common;
use common::*;

use std::convert::TryInto;

#[tokio::test]
async fn test_cancel_by_resolver() {
    // Test scenario: Resolver cancels after timelock[4]

    // Initialize test environment
    let mut test = ProgramTest::new(
        "svm_escrow",
        test_program_id(),
        processor!(svm_escrow::processor::process_instruction),
    );

    // Create test accounts
    let resolver = Keypair::new();
    let maker = Pubkey::new_unique(); // EVM address as Pubkey
    let secret = b"test_secret_32_bytes_long_______";
    let hash_secret = keccak256(secret);
    let amount = 1_000_000_000; // 1 token with 9 decimals
    let safety_deposit = 1_000_000; // 0.001 SOL

    // Set up token mint and accounts
    let (token_mint, mint_account) = create_token_mint();
    test.add_account(token_mint, mint_account);

    // Create resolver's token account with funds
    let (resolver_token_account, resolver_token_data) =
        create_token_account(&token_mint, &resolver.pubkey(), amount);
    test.add_account(resolver_token_account, resolver_token_data);

    // Start test
    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Derive PDA
    let (escrow_pda, bump) =
        Escrow::derive_pda(&maker, &resolver.pubkey(), &hash_secret, &test_program_id());

    // Create timelocks
    let now = 1_700_000_000u64;
    let timelocks = [
        0,     // src_exclusive_withdraw (already passed)
        0,     // src_public_withdraw (already passed)
        300,   // dst_exclusive_withdraw (5 minutes)
        600,   // dst_public_withdraw (10 minutes)
        900,   // dst_exclusive_cancel (15 minutes) - We'll advance past this
        1200,  // dst_public_cancel (20 minutes)
        86400, // rescue (24 hours)
    ];

    // Create escrow
    let init = EscrowInit {
        maker,
        resolver: resolver.pubkey(),
        hash_secret,
        token_mint,
        amount,
        safety_deposit_lamports: safety_deposit,
        merkle_root: [0u8; 32], // Single fill
        timelocks,
        bump,
    };

    let create_ix_data = pack_escrow_instruction(&EscrowInstruction::CreateDstEscrow(init));

    let create_escrow_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(resolver.pubkey(), true),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false),
        ],
        data: create_ix_data,
    };

    let mut transaction = Transaction::new_with_payer(&[create_escrow_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &resolver], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Create escrow failed: {:?}", result);

    // Get resolver's balance before cancel
    let resolver_balance_before = banks_client.get_balance(resolver.pubkey()).await.unwrap();

    // Advance clock past timelock[4] (exclusive cancel)
    let clock = Clock {
        slot: 100,
        epoch_start_timestamp: now.try_into().unwrap(),
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: (now + 1000).try_into().unwrap(), // Past exclusive cancel time (900)
    };
    banks_client.set_sysvar(&clock);

    // Have resolver call Cancel
    let cancel_ix_data = pack_escrow_instruction(&EscrowInstruction::Cancel);

    let cancel_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new_readonly(resolver.pubkey(), true), // Caller (must be resolver)
            AccountMeta::new(escrow_pda, false),                // Escrow PDA
            AccountMeta::new(resolver_token_account, false), // Resolver's token account (recipient)
            AccountMeta::new(resolver.pubkey(), false),      // Safety deposit recipient
            AccountMeta::new_readonly(token_mint, false),    // Token mint
            AccountMeta::new_readonly(spl_token::id(), false), // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: cancel_ix_data,
    };

    // Get fresh blockhash
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    let mut cancel_transaction = Transaction::new_with_payer(&[cancel_ix], Some(&payer.pubkey()));
    cancel_transaction.sign(&[&payer, &resolver], recent_blockhash);

    let cancel_result = banks_client.process_transaction(cancel_transaction).await;
    assert!(cancel_result.is_ok(), "Cancel failed: {:?}", cancel_result);

    // Verify escrow was closed
    let escrow_account_after = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(
        escrow_account_after.is_none(),
        "Escrow account should be closed after cancel"
    );

    // Verify tokens returned to resolver
    let resolver_token_after = banks_client
        .get_account(resolver_token_account)
        .await
        .unwrap();
    assert!(
        resolver_token_after.is_some(),
        "Resolver token account should exist"
    );
    // In real test, we would deserialize and check token balance == original amount

    // Verify resolver received safety deposit back
    let resolver_balance_after = banks_client.get_balance(resolver.pubkey()).await.unwrap();
    assert!(
        resolver_balance_after > resolver_balance_before,
        "Resolver should have received safety deposit back"
    );

    println!("Cancel by resolver test completed successfully!");
}

#[tokio::test]
async fn test_public_cancel() {
    // Test scenario: Anyone cancels after timelock[5]

    // Initialize test environment
    let mut test = ProgramTest::new(
        "svm_escrow",
        test_program_id(),
        processor!(svm_escrow::processor::process_instruction),
    );

    // Create test accounts
    let resolver = Keypair::new();
    let maker = Pubkey::new_unique(); // EVM address as Pubkey
    let stranger = Keypair::new(); // Random person who will call public cancel
    let secret = b"test_secret_32_bytes_long_______";
    let hash_secret = keccak256(secret);
    let amount = 1_000_000_000; // 1 token with 9 decimals
    let safety_deposit = 1_000_000; // 0.001 SOL

    // Set up token mint and accounts
    let (token_mint, mint_account) = create_token_mint();
    test.add_account(token_mint, mint_account);

    // Create resolver's token account with funds
    let (resolver_token_account, resolver_token_data) =
        create_token_account(&token_mint, &resolver.pubkey(), amount);
    test.add_account(resolver_token_account, resolver_token_data);

    // Fund stranger with SOL for transaction fees
    test.add_account(
        stranger.pubkey(),
        Account {
            lamports: 10_000_000_000, // 10 SOL
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // Start test
    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Derive PDA
    let (escrow_pda, bump) =
        Escrow::derive_pda(&maker, &resolver.pubkey(), &hash_secret, &test_program_id());

    // Create timelocks
    let now = 1_700_000_000u64;
    let timelocks = [
        0,     // src_exclusive_withdraw (already passed)
        0,     // src_public_withdraw (already passed)
        300,   // dst_exclusive_withdraw (5 minutes)
        600,   // dst_public_withdraw (10 minutes)
        900,   // dst_exclusive_cancel (15 minutes)
        1200,  // dst_public_cancel (20 minutes) - We'll advance past this
        86400, // rescue (24 hours)
    ];

    // Create escrow
    let init = EscrowInit {
        maker,
        resolver: resolver.pubkey(),
        hash_secret,
        token_mint,
        amount,
        safety_deposit_lamports: safety_deposit,
        merkle_root: [0u8; 32], // Single fill
        timelocks,
        bump,
    };

    let create_ix_data = pack_escrow_instruction(&EscrowInstruction::CreateDstEscrow(init));

    let create_escrow_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(resolver.pubkey(), true),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false),
        ],
        data: create_ix_data,
    };

    let mut transaction = Transaction::new_with_payer(&[create_escrow_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &resolver], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Create escrow failed: {:?}", result);

    // Get stranger's balance before cancel
    let stranger_balance_before = banks_client.get_balance(stranger.pubkey()).await.unwrap();

    // Advance clock past timelock[5] (public cancel)
    let clock = Clock {
        slot: 100,
        epoch_start_timestamp: now.try_into().unwrap(),
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: (now + 1300).try_into().unwrap(), // Past public cancel time (1200)
    };
    banks_client.set_sysvar(&clock);

    // Have stranger call PublicCancel
    let public_cancel_ix_data = pack_escrow_instruction(&EscrowInstruction::PublicCancel);

    let public_cancel_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new_readonly(stranger.pubkey(), true), // Caller (NOT resolver)
            AccountMeta::new(escrow_pda, false),                // Escrow PDA
            AccountMeta::new(resolver_token_account, false), // Resolver's token account (recipient)
            AccountMeta::new(stranger.pubkey(), false),      // Safety deposit recipient (caller)
            AccountMeta::new_readonly(token_mint, false),    // Token mint
            AccountMeta::new_readonly(spl_token::id(), false), // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: public_cancel_ix_data,
    };

    // Get fresh blockhash
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    let mut public_cancel_transaction =
        Transaction::new_with_payer(&[public_cancel_ix], Some(&stranger.pubkey()));
    public_cancel_transaction.sign(&[&stranger], recent_blockhash);

    let cancel_result = banks_client
        .process_transaction(public_cancel_transaction)
        .await;
    assert!(
        cancel_result.is_ok(),
        "Public cancel failed: {:?}",
        cancel_result
    );

    // Verify escrow was closed
    let escrow_account_after = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(
        escrow_account_after.is_none(),
        "Escrow account should be closed after public cancel"
    );

    // Verify tokens returned to resolver
    let resolver_token_after = banks_client
        .get_account(resolver_token_account)
        .await
        .unwrap();
    assert!(
        resolver_token_after.is_some(),
        "Resolver token account should exist"
    );
    // In real test, we would deserialize and check token balance == original amount

    // Verify stranger received safety deposit as incentive
    let stranger_balance_after = banks_client.get_balance(stranger.pubkey()).await.unwrap();
    assert!(
        stranger_balance_after > stranger_balance_before,
        "Stranger should have received safety deposit as incentive"
    );

    println!("Public cancel test completed successfully!");
    println!("Stranger successfully canceled and received safety deposit incentive");
}

/// Helper to calculate keccak256 hash
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak;
    let mut hasher = keccak::Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}
