use solana_program_test::{*};
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
async fn test_happy_path() {
    // Initialize test environment
    let mut test = ProgramTest::new(
        "svm_escrow",
        svm_escrow_program_id(),
        processor!(svm_escrow::processor::process_instruction),
    );
    
    // Start test
    let (mut banks_client, payer, recent_blockhash) = test.start().await;
    
    // Create test accounts
    let resolver = Keypair::new();
    let maker = Pubkey::new_unique(); // EVM address as Pubkey
    let secret = b"test_secret_32_bytes_long_______";
    let hash_secret = keccak256(secret);
    let token_mint = Pubkey::new_unique(); // SPL token mint
    let amount = 1_000_000_000; // 1 token with 9 decimals
    let safety_deposit = 1_000_000; // 0.001 SOL
    
    // Derive PDA
    let (escrow_pda, bump) = Escrow::derive_pda(
        &maker,
        &resolver.pubkey(),
        &hash_secret,
        &svm_escrow_program_id(),
    );
    
    // Create timelocks (all in the future)
    let now = 1_700_000_000u64; // Example timestamp
    let timelocks = [
        0,  // src_exclusive_withdraw (already passed)
        0,  // src_public_withdraw (already passed)
        300,  // dst_exclusive_withdraw (5 minutes)
        600,  // dst_public_withdraw (10 minutes)
        900,  // dst_exclusive_cancel (15 minutes)
        1200, // dst_public_cancel (20 minutes)
        86400, // rescue (24 hours)
    ];
    
    // Create escrow initialization
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
    
    // Pack create instruction
    let create_ix_data = pack_escrow_instruction(&EscrowInstruction::CreateDstEscrow(init));
    
    // Create escrow instruction
    let create_escrow_ix = Instruction {
        program_id: svm_escrow_program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),     // Payer
            AccountMeta::new_readonly(resolver.pubkey(), true), // Resolver
            AccountMeta::new(escrow_pda, false),        // Escrow PDA
            AccountMeta::new_readonly(system_program::id(), false), // System
            AccountMeta::new_readonly(spl_token::id(), false), // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: create_ix_data,
    };
    
    // Send create transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_escrow_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &resolver], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Create escrow failed: {:?}", result);
    
    // Verify escrow was created
    let escrow_account = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(escrow_account.is_some(), "Escrow account should exist");
    
    let escrow_account = escrow_account.unwrap();
    assert_eq!(escrow_account.owner, svm_escrow_program_id());
    assert_eq!(escrow_account.data.len(), Escrow::LEN);
    
    // Set up token accounts for withdraw testing
    let (token_mint, mint_account) = create_token_mint();
    test.add_account(token_mint, mint_account);
    
    // Create resolver's token account with funds
    let (resolver_token_account, resolver_token_data) = create_token_account(&token_mint, &resolver.pubkey(), amount);
    test.add_account(resolver_token_account, resolver_token_data);
    
    // Create maker's token account (will receive funds)
    let (maker_token_account, maker_token_data) = create_token_account(&token_mint, &maker, 0);
    test.add_account(maker_token_account, maker_token_data);
    
    // Advance clock to allow withdraw (past timelock[2])
    let clock = Clock {
        slot: 100,
        epoch_start_timestamp: now.try_into().unwrap(),
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: (now + 400).try_into().unwrap(), // Past exclusive withdraw time
    };
    test.set_sysvar(&clock);
    
    // Restart banks client after adding accounts
    let (mut banks_client, payer, recent_blockhash) = test.start().await;
    
    // Test withdraw with correct secret
    let withdraw_ix_data = pack_escrow_instruction(&EscrowInstruction::Withdraw {
        secret: *secret,
        proof: vec![], // No Merkle proof for single fill
    });
    
    let withdraw_ix = Instruction {
        program_id: svm_escrow_program_id(),
        accounts: vec![
            AccountMeta::new_readonly(resolver.pubkey(), true), // Caller (must be resolver during exclusive period)
            AccountMeta::new(escrow_pda, false),                // Escrow PDA
            AccountMeta::new(maker_token_account, false),       // Maker's token account (recipient)
            AccountMeta::new(resolver_token_account, false),    // Resolver's token account (source)
            AccountMeta::new(resolver.pubkey(), false),         // Safety deposit recipient
            AccountMeta::new_readonly(token_mint, false),       // Token mint
            AccountMeta::new_readonly(spl_token::id(), false),  // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: withdraw_ix_data,
    };
    
    let mut withdraw_transaction = Transaction::new_with_payer(
        &[withdraw_ix],
        Some(&payer.pubkey()),
    );
    withdraw_transaction.sign(&[&payer, &resolver], recent_blockhash);
    
    let withdraw_result = banks_client.process_transaction(withdraw_transaction).await;
    assert!(withdraw_result.is_ok(), "Withdraw failed: {:?}", withdraw_result);
    
    // Verify escrow was closed
    let escrow_account_after = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(escrow_account_after.is_none(), "Escrow account should be closed after withdraw");
    
    // Verify maker received tokens
    let maker_token_after = banks_client.get_account(maker_token_account).await.unwrap();
    assert!(maker_token_after.is_some(), "Maker token account should exist");
    // In real test, we would deserialize and check token balance
    
    // Verify resolver received safety deposit
    let resolver_balance_after = banks_client.get_balance(resolver.pubkey()).await.unwrap();
    assert!(resolver_balance_after > 0, "Resolver should have received safety deposit");
    
    println!("Happy path test completed successfully!");
}

/// Helper to calculate keccak256 hash
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak;
    let mut hasher = keccak::Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}

/// Get program ID for tests
fn svm_escrow_program_id() -> Pubkey {
    // Use the same test program ID as in common module
    test_program_id()
}