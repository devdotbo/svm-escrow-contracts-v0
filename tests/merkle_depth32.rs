use solana_program_test::*;
use solana_sdk::{
    account::Account,
    clock::Clock,
    compute_budget::ComputeBudgetInstruction,
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
async fn test_merkle_depth_32() {
    // Test scenario: 32-level Merkle proof verification

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

    // Build a Merkle tree with the secret hash as one of the leaves
    // For testing, we'll simulate a tree with our leaf at index 42
    let leaf_index = 42;
    let merkle_data = build_merkle_data_for_32_levels(&hash_secret, leaf_index);
    let merkle_root = merkle_data.root;
    let merkle_proof = merkle_data.proof;

    assert_eq!(
        merkle_proof.len(),
        32,
        "Proof should have exactly 32 elements"
    );

    // Set up token mint and accounts
    let (token_mint, mint_account) = create_token_mint();
    test.add_account(token_mint, mint_account);

    // Create resolver's token account with funds
    let (resolver_token_account, resolver_token_data) =
        create_token_account(&token_mint, &resolver.pubkey(), amount);
    test.add_account(resolver_token_account, resolver_token_data);

    // Create maker's token account (will receive funds)
    let (maker_token_account, maker_token_data) = create_token_account(&token_mint, &maker, 0);
    test.add_account(maker_token_account, maker_token_data);

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
        1200,  // dst_public_cancel (20 minutes)
        86400, // rescue (24 hours)
    ];

    // Create escrow with Merkle root
    let init = EscrowInit {
        maker,
        resolver: resolver.pubkey(),
        hash_secret,
        token_mint,
        amount,
        safety_deposit_lamports: safety_deposit,
        merkle_root, // Using our generated Merkle root
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

    // Advance clock to allow withdraw
    let clock = Clock {
        slot: 100,
        epoch_start_timestamp: now.try_into().unwrap(),
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: (now + 400).try_into().unwrap(), // Past exclusive withdraw time
    };
    banks_client.set_sysvar(&clock);

    // Get fresh blockhash
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // Execute withdraw with deep proof
    // Add compute budget instruction to ensure we have enough CUs
    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(300_000); // Allow up to 300k CUs

    let withdraw_ix_data = pack_escrow_instruction(&EscrowInstruction::Withdraw {
        secret: *secret,
        proof: merkle_proof, // 32-level proof
    });

    let withdraw_ix = Instruction {
        program_id: test_program_id(),
        accounts: vec![
            AccountMeta::new_readonly(resolver.pubkey(), true), // Caller
            AccountMeta::new(escrow_pda, false),                // Escrow PDA
            AccountMeta::new(maker_token_account, false),       // Maker's token account
            AccountMeta::new(resolver_token_account, false),    // Resolver's token account
            AccountMeta::new(resolver.pubkey(), false),         // Safety deposit recipient
            AccountMeta::new_readonly(token_mint, false),       // Token mint
            AccountMeta::new_readonly(spl_token::id(), false),  // Token program
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false), // Clock
        ],
        data: withdraw_ix_data,
    };

    let mut withdraw_transaction =
        Transaction::new_with_payer(&[compute_budget_ix, withdraw_ix], Some(&payer.pubkey()));
    withdraw_transaction.sign(&[&payer, &resolver], recent_blockhash);

    let withdraw_result = banks_client.process_transaction(withdraw_transaction).await;
    assert!(
        withdraw_result.is_ok(),
        "Withdraw with 32-level proof failed: {:?}",
        withdraw_result
    );

    // Verify escrow was closed
    let escrow_account_after = banks_client.get_account(escrow_pda).await.unwrap();
    assert!(
        escrow_account_after.is_none(),
        "Escrow account should be closed after withdraw"
    );

    // In a real test environment, we would measure actual CU usage
    // For now, we just verify the transaction succeeded
    println!("Merkle depth 32 test completed successfully!");
    println!("32-level Merkle proof verified within CU budget");
}

/// Helper to calculate keccak256 hash
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak;
    let mut hasher = keccak::Hasher::default();
    hasher.hash(data);
    hasher.result().to_bytes()
}

/// Structure to hold Merkle tree data
struct MerkleData {
    root: [u8; 32],
    proof: Vec<[u8; 32]>,
}

/// Helper to build Merkle data for testing 32-level proofs
fn build_merkle_data_for_32_levels(leaf_hash: &[u8; 32], leaf_index: usize) -> MerkleData {
    use solana_program::keccak;

    // For a 32-level tree, we need a proof path from leaf to root
    // We'll simulate this by generating sibling hashes at each level
    let mut proof = Vec::with_capacity(32);
    let mut current_hash = *leaf_hash;
    let mut current_index = leaf_index;

    // Generate 32 levels of proof
    for level in 0..32 {
        // Generate a deterministic sibling hash for testing
        let sibling_hash = {
            let mut sibling = [0u8; 32];
            sibling[0] = level as u8;
            sibling[1] = (level + 1) as u8;
            sibling[2] = 0xAB; // Marker byte
            keccak::hashv(&[&sibling]).to_bytes()
        };

        proof.push(sibling_hash);

        // Compute parent hash
        let (left, right) = if current_index % 2 == 0 {
            (&current_hash, &sibling_hash)
        } else {
            (&sibling_hash, &current_hash)
        };

        current_hash = keccak::hashv(&[left, right]).to_bytes();
        current_index /= 2;
    }

    MerkleData {
        root: current_hash,
        proof,
    }
}
