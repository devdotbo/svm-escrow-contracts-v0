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
/// Accounts expected:
/// 0. [WRITE] Escrow PDA account
/// 1. [WRITE] Maker token account (destination)
/// 2. [WRITE] Escrow token account (source)
/// 3. [WRITE] Caller account (receives safety deposit)
/// 4. [] Token program
/// 5. [] Clock sysvar
/// 6. [] System program
/// 7. [WRITE, optional] Target account (for WithdrawTo)
fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    secret: [u8; 32],
    proof: Vec<[u8; 32]>,
    target: Option<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let escrow_info = next_account_info(account_info_iter)?;
    let maker_token_info = next_account_info(account_info_iter)?;
    let escrow_token_info = next_account_info(account_info_iter)?;
    let caller_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    
    // Get target account if WithdrawTo
    let target_token_info = if target.is_some() {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };
    
    // Load and verify escrow
    let escrow = Escrow::from_account_info(escrow_info)?;
    if escrow_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify PDA
    let (expected_pda, _bump) = Escrow::derive_pda(
        &escrow.maker,
        &escrow.resolver,
        &escrow.hash_secret,
        program_id,
    );
    if escrow_info.key != &expected_pda {
        return Err(EscrowError::InvalidPDA.into());
    }
    
    // Verify secret using keccak256 (260 CU as per spec)
    let computed_hash = {
        use solana_program::keccak;
        keccak::hashv(&[&secret]).to_bytes()
    };
    
    if computed_hash != escrow.hash_secret {
        return Err(EscrowError::InvalidSecret.into());
    }
    
    // Get current time
    let clock = Clock::from_account_info(clock_info)?;
    let current_time = clock.unix_timestamp as u64;
    
    // Check timelocks
    let exclusive_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_DST_EXCLUSIVE_WITHDRAW];
    let public_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_DST_PUBLIC_WITHDRAW];
    
    if current_time < exclusive_time {
        return Err(EscrowError::TimelockNotExpired.into());
    }
    
    // During exclusive phase, only resolver can withdraw
    if current_time < public_time && caller_info.key != &escrow.resolver {
        return Err(EscrowError::Unauthorized.into());
    }
    
    // Verify Merkle proof if needed
    if escrow.merkle_root != [0u8; 32] {
        verify_merkle_proof(&secret, &proof, &escrow.merkle_root, escrow.filled_index)?;
        
        // Increment filled index (will be saved later)
        let escrow_mut = Escrow::from_account_info_mut(escrow_info)?;
        escrow_mut.filled_index = escrow_mut.filled_index
            .checked_add(1)
            .ok_or(EscrowError::Overflow)?;
    }
    
    // Determine destination token account
    let destination_token_info = if let Some(ref target_account) = target_token_info {
        target_account
    } else {
        maker_token_info
    };
    
    // Transfer tokens from escrow to destination
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        escrow_token_info.key,
        destination_token_info.key,
        escrow_info.key,
        &[],
        escrow.amount,
    )?;
    
    let seeds = &[
        Escrow::SEED_PREFIX,
        escrow.maker.as_ref(),
        escrow.resolver.as_ref(),
        &escrow.hash_secret,
        &[Escrow::ROLE_BYTE_DST],
        &[escrow.bump],
    ];
    
    invoke_signed(
        &transfer_ix,
        &[
            escrow_token_info.clone(),
            destination_token_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[seeds],
    )?;
    
    // Transfer safety deposit to caller (last to prevent re-entrancy)
    **escrow_info.try_borrow_mut_lamports()? -= escrow.safety_deposit_lamports;
    **caller_info.try_borrow_mut_lamports()? += escrow.safety_deposit_lamports;
    
    // Close PDA if single-fill or last Merkle leaf
    let should_close = escrow.merkle_root == [0u8; 32] || 
                      escrow.filled_index >= (1u32 << 32); // Max 2^32 leaves
    
    if should_close {
        // Transfer remaining lamports to caller and zero the account
        let remaining_lamports = escrow_info.lamports();
        **escrow_info.try_borrow_mut_lamports()? = 0;
        **caller_info.try_borrow_mut_lamports()? += remaining_lamports;
    }
    
    solana_program::msg!("Withdraw successful for escrow {}", escrow_info.key);
    
    Ok(())
}

/// Verify Merkle proof for batch fills
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn verify_merkle_proof(
    leaf: &[u8; 32],
    proof: &[[u8; 32]],
    root: &[u8; 32],
    index: u32,
) -> ProgramResult {
    if proof.len() > 32 {
        return Err(EscrowError::MerkleProofTooDeep.into());
    }
    
    let mut computed_hash = *leaf;
    let mut idx = index;
    
    for (i, sibling) in proof.iter().enumerate() {
        if i >= 32 {
            // Hard cap at 32 levels as per spec
            break;
        }
        
        computed_hash = if idx & 1 == 0 {
            // Current node is left child
            hash_pair(&computed_hash, sibling)
        } else {
            // Current node is right child
            hash_pair(sibling, &computed_hash)
        };
        
        idx >>= 1;
    }
    
    if computed_hash != *root {
        return Err(EscrowError::InvalidMerkleProof.into());
    }
    
    Ok(())
}

/// Hash two nodes for Merkle tree
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    use solana_program::keccak;
    let mut data = [0u8; 64];
    data[..32].copy_from_slice(left);
    data[32..].copy_from_slice(right);
    keccak::hashv(&[&data]).to_bytes()
}

/// Process Cancel/PublicCancel instructions
/// Accounts expected:
/// 0. [WRITE] Escrow PDA account
/// 1. [WRITE] Resolver token account (destination)
/// 2. [WRITE] Escrow token account (source)
/// 3. [WRITE] Caller account (receives safety deposit)
/// 4. [] Token program
/// 5. [] Clock sysvar
/// 6. [] System program
fn process_cancel(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_public: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let escrow_info = next_account_info(account_info_iter)?;
    let resolver_token_info = next_account_info(account_info_iter)?;
    let escrow_token_info = next_account_info(account_info_iter)?;
    let caller_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    
    // Load and verify escrow
    let escrow = Escrow::from_account_info(escrow_info)?;
    if escrow_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify PDA
    let (expected_pda, _bump) = Escrow::derive_pda(
        &escrow.maker,
        &escrow.resolver,
        &escrow.hash_secret,
        program_id,
    );
    if escrow_info.key != &expected_pda {
        return Err(EscrowError::InvalidPDA.into());
    }
    
    // Get current time
    let clock = Clock::from_account_info(clock_info)?;
    let current_time = clock.unix_timestamp as u64;
    
    // Check timelocks
    let exclusive_cancel_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_DST_EXCLUSIVE_CANCEL];
    let public_cancel_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_DST_PUBLIC_CANCEL];
    
    if is_public {
        // PublicCancel - anyone can call after public cancel time
        if current_time < public_cancel_time {
            return Err(EscrowError::TimelockNotExpired.into());
        }
    } else {
        // Cancel - only resolver during exclusive period
        if current_time < exclusive_cancel_time {
            return Err(EscrowError::TimelockNotExpired.into());
        }
        
        // During exclusive phase, only resolver can cancel
        if current_time < public_cancel_time && caller_info.key != &escrow.resolver {
            return Err(EscrowError::Unauthorized.into());
        }
    }
    
    // Transfer tokens back to resolver
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        escrow_token_info.key,
        resolver_token_info.key,
        escrow_info.key,
        &[],
        escrow.amount,
    )?;
    
    let seeds = &[
        Escrow::SEED_PREFIX,
        escrow.maker.as_ref(),
        escrow.resolver.as_ref(),
        &escrow.hash_secret,
        &[Escrow::ROLE_BYTE_DST],
        &[escrow.bump],
    ];
    
    invoke_signed(
        &transfer_ix,
        &[
            escrow_token_info.clone(),
            resolver_token_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[seeds],
    )?;
    
    // Transfer safety deposit to caller as incentive (last to prevent re-entrancy)
    **escrow_info.try_borrow_mut_lamports()? -= escrow.safety_deposit_lamports;
    **caller_info.try_borrow_mut_lamports()? += escrow.safety_deposit_lamports;
    
    // Close PDA - transfer remaining lamports to caller
    let remaining_lamports = escrow_info.lamports();
    **escrow_info.try_borrow_mut_lamports()? = 0;
    **caller_info.try_borrow_mut_lamports()? += remaining_lamports;
    
    solana_program::msg!("Cancel successful for escrow {}", escrow_info.key);
    
    Ok(())
}

/// Process PublicWithdraw instruction
/// Similar to Withdraw but enforces public phase timing
fn process_public_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    secret: [u8; 32],
    proof: Vec<[u8; 32]>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let escrow_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    
    // Load escrow to check public withdraw timelock
    let escrow = Escrow::from_account_info(escrow_info)?;
    let clock = Clock::from_account_info(clock_info)?;
    let current_time = clock.unix_timestamp as u64;
    
    // Ensure we're in public withdraw phase
    let public_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_DST_PUBLIC_WITHDRAW];
    
    if current_time < public_time {
        return Err(EscrowError::TimelockNotExpired.into());
    }
    
    // Delegate to regular withdraw (which will allow anyone since we're past public time)
    process_withdraw(program_id, accounts, secret, proof, None)
}

/// Process RescueFunds instruction
/// Accounts expected:
/// 0. [WRITE] Escrow PDA account
/// 1. [WRITE] Resolver account (receives funds)
/// 2. [WRITE] Escrow token account (if any tokens remain)
/// 3. [] Token program
/// 4. [] Clock sysvar
/// 5. [] System program
fn process_rescue_funds(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let escrow_info = next_account_info(account_info_iter)?;
    let resolver_info = next_account_info(account_info_iter)?;
    let escrow_token_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;
    
    // Load and verify escrow
    let escrow = Escrow::from_account_info(escrow_info)?;
    if escrow_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Verify PDA
    let (expected_pda, _bump) = Escrow::derive_pda(
        &escrow.maker,
        &escrow.resolver,
        &escrow.hash_secret,
        program_id,
    );
    if escrow_info.key != &expected_pda {
        return Err(EscrowError::InvalidPDA.into());
    }
    
    // Get current time
    let clock = Clock::from_account_info(clock_info)?;
    let current_time = clock.unix_timestamp as u64;
    
    // Check rescue timelock
    let rescue_time = escrow.deployed_at + escrow.timelocks[Escrow::TL_RESCUE];
    
    if current_time < rescue_time {
        return Err(EscrowError::TimelockNotExpired.into());
    }
    
    // If there are any tokens remaining, transfer them to resolver
    if escrow.amount > 0 {
        // Get resolver token account from the provided account
        let resolver_token_info = next_account_info(account_info_iter)?;
        
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            escrow_token_info.key,
            resolver_token_info.key,
            escrow_info.key,
            &[],
            escrow.amount,
        )?;
        
        let seeds = &[
            Escrow::SEED_PREFIX,
            escrow.maker.as_ref(),
            escrow.resolver.as_ref(),
            &escrow.hash_secret,
            &[Escrow::ROLE_BYTE_DST],
            &[escrow.bump],
        ];
        
        invoke_signed(
            &transfer_ix,
            &[
                escrow_token_info.clone(),
                resolver_token_info.clone(),
                escrow_info.clone(),
                token_program_info.clone(),
            ],
            &[seeds],
        )?;
    }
    
    // Transfer all remaining lamports to resolver and close PDA
    let remaining_lamports = escrow_info.lamports();
    **escrow_info.try_borrow_mut_lamports()? = 0;
    **resolver_info.try_borrow_mut_lamports()? += remaining_lamports;
    
    solana_program::msg!("Rescue funds successful for escrow {}", escrow_info.key);
    
    Ok(())
}