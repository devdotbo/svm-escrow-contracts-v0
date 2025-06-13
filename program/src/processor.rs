use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
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
fn process_create_dst_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    init: EscrowInit,
) -> ProgramResult {
    // TODO: Implement
    solana_program::msg!("CreateDstEscrow instruction");
    Ok(())
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