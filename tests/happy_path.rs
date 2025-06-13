use solana_program_test::{*};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use svm_escrow::{
    instruction::{EscrowInit, EscrowInstruction},
    state::Escrow,
};

mod common;
use common::*;

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
    
    // TODO: Pack instruction data
    // TODO: Create and send transaction
    // TODO: Verify escrow was created correctly
    // TODO: Test withdraw with secret
    // TODO: Verify tokens transferred and PDA closed
    
    println!("Happy path test - TODO: Complete implementation");
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
    // This should match the actual program ID
    Pubkey::new_unique()
}