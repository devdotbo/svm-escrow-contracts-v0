use solana_program::pubkey::Pubkey;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    system_program,
};

/// Create SPL token mint account for testing
pub fn create_token_mint() -> (Pubkey, Account) {
    let mint_pubkey = Pubkey::new_unique();
    let mint_account = Account {
        lamports: 1_000_000_000,
        data: vec![0; 82], // SPL Token Mint size
        owner: spl_token::id(),
        executable: false,
        rent_epoch: 0,
    };
    (mint_pubkey, mint_account)
}

/// Create SPL token account for testing
pub fn create_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> (Pubkey, Account) {
    let account_pubkey = Pubkey::new_unique();
    let mut data = vec![0; 165]; // SPL Token Account size
    
    // Mock token account data (simplified)
    // In real tests, use spl_token::state::Account
    
    let account = Account {
        lamports: 2_039_280, // Rent-exempt amount
        data,
        owner: spl_token::id(),
        executable: false,
        rent_epoch: 0,
    };
    (account_pubkey, account)
}

/// Pack escrow instruction for testing
pub fn pack_escrow_instruction(instruction: &svm_escrow::instruction::EscrowInstruction) -> Vec<u8> {
    use svm_escrow::instruction::{EscrowInstruction, EscrowInit};
    use std::mem;
    
    match instruction {
        EscrowInstruction::CreateDstEscrow(init) => {
            let mut data = vec![0u8]; // Discriminator
            // Pack the struct directly as bytes (matching on-chain expectation)
            let init_bytes = unsafe {
                std::slice::from_raw_parts(
                    init as *const EscrowInit as *const u8,
                    mem::size_of::<EscrowInit>(),
                )
            };
            data.extend_from_slice(init_bytes);
            data
        }
        EscrowInstruction::Withdraw { secret, proof } => {
            let mut data = vec![1u8]; // Discriminator
            data.extend_from_slice(secret);
            for p in proof {
                data.extend_from_slice(p);
            }
            data
        }
        EscrowInstruction::WithdrawTo { secret, target, proof } => {
            let mut data = vec![2u8]; // Discriminator
            data.extend_from_slice(secret);
            data.extend_from_slice(target.as_ref());
            for p in proof {
                data.extend_from_slice(p);
            }
            data
        }
        EscrowInstruction::Cancel => vec![3u8],
        EscrowInstruction::PublicCancel => vec![4u8],
        EscrowInstruction::PublicWithdraw { secret, proof } => {
            let mut data = vec![5u8]; // Discriminator
            data.extend_from_slice(secret);
            for p in proof {
                data.extend_from_slice(p);
            }
            data
        }
        EscrowInstruction::RescueFunds => vec![6u8],
    }
}

/// Get test program ID
pub fn test_program_id() -> Pubkey {
    // Use a deterministic program ID for tests
    Pubkey::new_from_array([1u8; 32])
}