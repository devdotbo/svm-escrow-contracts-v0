#![cfg_attr(not(feature = "no-entrypoint"), no_std)]
#![cfg_attr(not(feature = "no-entrypoint"), no_main)]

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

#[cfg(not(feature = "no-entrypoint"))]
use processor::process_instruction;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process);

#[cfg(not(feature = "no-entrypoint"))]
fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    process_instruction(program_id, accounts, instruction_data)
}

// Export types for use by clients
pub use error::EscrowError;
pub use instruction::{EscrowInit, EscrowInstruction};
pub use state::Escrow;