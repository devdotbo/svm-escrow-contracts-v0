use solana_program_test::{*};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    transaction::Transaction,
};
use svm_escrow::{
    instruction::{EscrowInit, EscrowInstruction},
    state::Escrow,
};

mod common;
use common::*;

#[tokio::test]
async fn test_cu_budget_create() {
    // Test CU usage for CreateDstEscrow
    
    // TODO: Create transaction with compute budget instruction
    // TODO: Execute CreateDstEscrow
    // TODO: Measure actual CU used
    // TODO: Assert CU < 200k
    
    println!("CU budget create test - TODO: Complete implementation");
}

#[tokio::test]
async fn test_cu_budget_withdraw_merkle() {
    // Test CU usage for Withdraw with 32-level Merkle proof
    
    // TODO: Create escrow with Merkle root
    // TODO: Create transaction with compute budget instruction
    // TODO: Execute Withdraw with 32-level proof
    // TODO: Measure actual CU used
    // TODO: Assert CU < 200k (spec requirement)
    
    println!("CU budget withdraw with Merkle test - TODO: Complete implementation");
}

#[tokio::test]
async fn test_cu_budget_all_instructions() {
    // Test CU usage for all instructions
    let instructions = vec![
        "CreateDstEscrow",
        "Withdraw",
        "WithdrawTo",
        "Cancel",
        "PublicCancel",
        "PublicWithdraw",
        "RescueFunds",
    ];
    
    for ix in instructions {
        // TODO: Test each instruction's CU usage
        println!("Testing CU for {}: TODO", ix);
    }
}