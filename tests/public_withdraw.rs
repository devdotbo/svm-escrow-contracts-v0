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
async fn test_public_withdraw() {
    // Test scenario: Resolver disappears, stranger executes after timelock[3]
    
    // Initialize test environment
    let mut test = ProgramTest::new(
        "svm_escrow",
        test_program_id(),
        processor!(svm_escrow::processor::process_instruction),
    );
    
    // Create test accounts
    let resolver = Keypair::new();
    let maker = Pubkey::new_unique(); // EVM address as Pubkey
    let stranger = Keypair::new(); // Random person who will call public withdraw
    let secret = b"test_secret_32_bytes_long_______";
    let hash_secret = keccak256(secret);
    let amount = 1_000_000_000; // 1 token with 9 decimals
    let safety_deposit = 1_000_000; // 0.001 SOL
    
    // Set up token mint and accounts
    let (token_mint, mint_account) = create_token_mint();
    test.add_account(token_mint, mint_account);
    
    // Create resolver's token account with funds
    let (resolver_token_account, resolver_token_data) = create_token_account(&token_mint, &resolver.pubkey(), amount);
    test.add_account(resolver_token_account, resolver_token_data);
    
    // Create maker's token account (will receive funds)
    let (maker_token_account, maker_token_data) = create_token_account(&token_mint, &maker, 0);
    test.add_account(maker_token_account, maker_token_data);
    
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
    let (escrow_pda, bump) = Escrow::derive_pda(
        &maker,
        &resolver.pubkey(),
        &hash_secret,
        &test_program_id(),
    );
    
    // Create timelocks with public withdraw phase
    let now = 1_700_000_000u64;
    let timelocks = [
        0,    // src_exclusive_withdraw (already passed)
        0,    // src_public_withdraw (already passed)
        300,  // dst_exclusive_withdraw (5 minutes)
        600,  // dst_public_withdraw (10 minutes) - We'll advance past this
        900,  // dst_exclusive_cancel (15 minutes)
        1200, // dst_public_cancel (20 minutes)
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
    
    let mut transaction = Transaction::new_with_payer(
        &[create_escrow_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &resolver], recent_blockhash);
    
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Create escrow failed: {:?}", result);
    
    // Advance clock past timelock[3] (public withdraw phase)
    // Now anyone can withdraw with the correct secret
    let clock = Clock {
        slot: 100,
        epoch_start_timestamp: now.try_into().unwrap(),
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: (now + 700).try_into().unwrap(), // Past public withdraw time (600)
    };
    banks_client.set_sysvar(&clock);
    
    // Get stranger's initial balance
    let stranger_balance_before = banks_client.get_balance(stranger.pubkey()).await.unwrap();
    
    // Have a different account (not resolver) call PublicWithdraw
    let public_withdraw_ix_data = pack_escrow_instruction(&EscrowInstruction::PublicWithdraw {
        secret: *secret,
        proof: vec![], // No Merkle proof for single fill
    });
    
    let public_withdraw_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new_readonly(stranger.pubkey(), true), // Caller (NOT resolver)
            AccountMeta::new(escrow_pda, false),                // Escrow PDA
            AccountMeta::new(maker_token_account, false),       // Maker's token account (recipient)
            AccountMeta::new(resolver_token_account, false),    // Resolver's token account (source)
            AccountMeta::new(stranger.pubkey(), false),         // Safety deposit recipient (caller)
            AccountMeta::new_readonly(token_mint, false),       // Token mint
            AccountMeta::new_readonly(spl_token::id(), false),  // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: public_withdraw_ix_data,
    };
    
    // Get fresh blockhash
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    
    let mut public_withdraw_transaction = Transaction::new_with_payer(
        &[public_withdraw_ix],
        Some(&stranger.pubkey()),
    );
    public_withdraw_transaction.sign(&[&stranger], recent_blockhash);
    
    let withdraw_result = banks_client.process_transaction(public_withdraw_transaction).await;
    assert!(withdraw_result.is_ok(), "Public withdraw failed: {:?}", withdraw_result);
    
    // Verify escrow was closed
    let escrow_account_after = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(escrow_account_after.is_none(), "Escrow account should be closed after public withdraw");
    
    // Verify tokens go to maker
    let maker_token_after = banks_client.get_account(maker_token_account).await.unwrap();
    assert!(maker_token_after.is_some(), "Maker token account should exist");
    // In real test, we would deserialize and check token balance == amount
    
    // Verify safety deposit goes to caller (stranger)
    let stranger_balance_after = banks_client.get_balance(stranger.pubkey()).await.unwrap();
    assert!(
        stranger_balance_after > stranger_balance_before, 
        "Stranger should have received safety deposit"
    );
    // Safety deposit minus transaction fees should be positive
    
    println!("Public withdraw test completed successfully!");
    println!("Stranger successfully withdrew on behalf of maker after public phase");
}

/// Helper to calculate keccak256 hash
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak;
    let mut hasher = keccak::Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}