use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
    ProgramError,
};

use crate::{
    error::EscrowError,
    instruction::{EscrowInit, EscrowInstruction},
    state::Escrow,
};

/// Program entrypoint processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;
    
    match instruction {
        EscrowInstruction::CreateDstEscrow(init) => {
            process_create_dst_escrow(program_id, accounts, init)
        }
        EscrowInstruction::Withdraw { secret, proof } => {
            process_withdraw(program_id, accounts, secret, proof, None)
        }
        EscrowInstruction::WithdrawTo { secret, target, proof } => {
            process_withdraw(program_id, accounts, secret, proof, Some(target))
        }
        EscrowInstruction::Cancel => {
            process_cancel(program_id, accounts, false)
        }
        EscrowInstruction::PublicCancel => {
            process_cancel(program_id, accounts, true)
        }
        EscrowInstruction::PublicWithdraw { secret, proof } => {
            process_public_withdraw(program_id, accounts, secret, proof)
        }
        EscrowInstruction::RescueFunds => {
            process_rescue_funds(program_id, accounts)
        }
    }
}

/// Process CreateDstEscrow instruction
/// Accounts expected:
/// 0. [WRITE, SIGNER] Payer account
/// 1. [SIGNER] Resolver account
/// 2. [WRITE] Escrow PDA account
/// 3. [] System Program
/// 4. [] Token program
/// 5. [] Clock sysvar
fn process_create_dst_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    init: EscrowInit,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let payer_info = next_account_info(account_info_iter)?;
    let resolver_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let _token_program_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    
    // Verify signers
    if !payer_info.is_signer {
        return Err(EscrowError::Unauthorized.into());
    }
    if !resolver_info.is_signer {
        return Err(EscrowError::Unauthorized.into());
    }
    
    // Verify resolver matches the init param
    if resolver_info.key != &init.resolver {
        return Err(EscrowError::Unauthorized.into());
    }
    
    // Derive PDA and verify it matches the provided account
    let (expected_pda, bump) = Escrow::derive_pda(
        &init.maker,
        &init.resolver,
        &init.hash_secret,
        program_id,
    );
    
    if escrow_info.key != &expected_pda {
        return Err(EscrowError::InvalidPDA.into());
    }
    
    // Verify bump matches
    if bump != init.bump {
        return Err(EscrowError::InvalidPDA.into());
    }
    
    // Ensure PDA is uninitialized (zero lamports)
    if escrow_info.lamports() > 0 {
        return Err(EscrowError::AlreadyInitialized.into());
    }
    
    // Get rent and clock
    let rent = Rent::get()?;
    let clock = Clock::from_account_info(clock_info)?;
    
    // Calculate required lamports (rent-exempt + safety deposit)
    let rent_exempt_lamports = rent.minimum_balance(Escrow::LEN);
    let total_lamports = rent_exempt_lamports
        .checked_add(init.safety_deposit_lamports)
        .ok_or(EscrowError::Overflow)?;
    
    // Create PDA account
    let seeds = &[
        Escrow::SEED_PREFIX,
        init.maker.as_ref(),
        init.resolver.as_ref(),
        &init.hash_secret,
        &[Escrow::ROLE_BYTE_DST],
        &[bump],
    ];
    
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            escrow_info.key,
            total_lamports,
            Escrow::LEN as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            escrow_info.clone(),
            system_program_info.clone(),
        ],
        &[seeds],
    )?;
    
    // Initialize escrow data
    let escrow = Escrow::from_account_info_mut(escrow_info)?;
    escrow.maker = init.maker;
    escrow.resolver = init.resolver;
    escrow.hash_secret = init.hash_secret;
    escrow.token_mint = init.token_mint;
    escrow.amount = init.amount;
    escrow.safety_deposit_lamports = init.safety_deposit_lamports;
    escrow.merkle_root = init.merkle_root;
    escrow.filled_index = 0;
    escrow.timelocks = init.timelocks;
    escrow.deployed_at = clock.unix_timestamp as u64;
    escrow.bump = bump;
    
    solana_program::msg!("Escrow created at {}", escrow_info.key);
    
    Ok(())
}

/// Helper function to get next account
fn next_account_info<'a, 'b: 'a>(
    iter: &mut std::slice::Iter<'a, AccountInfo<'b>>,
) -> Result<&'a AccountInfo<'b>, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

/// Process Withdraw/WithdrawTo instructions
fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    secret: [u8; 32],
    proof: Vec<[u8; 32]>,
    target: Option<Pubkey>,
) -> ProgramResult {
    // TODO: Implement
    solana_program::msg!("Withdraw instruction");
    Ok(())
}

/// Process Cancel/PublicCancel instructions
fn process_cancel(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_public: bool,
) -> ProgramResult {
    // TODO: Implement
    solana_program::msg!("Cancel instruction");
    Ok(())
}

/// Process PublicWithdraw instruction
fn process_public_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    secret: [u8; 32],
    proof: Vec<[u8; 32]>,
) -> ProgramResult {
    // TODO: Implement
    solana_program::msg!("PublicWithdraw instruction");
    Ok(())
}

/// Process RescueFunds instruction
fn process_rescue_funds(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // TODO: Implement
    solana_program::msg!("RescueFunds instruction");
    Ok(())
}