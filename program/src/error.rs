use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Custom errors for the escrow program
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum EscrowError {
    #[error("Invalid instruction")]
    InvalidInstruction = 0,

    #[error("Invalid PDA")]
    InvalidPDA = 1,

    #[error("Escrow already initialized")]
    AlreadyInitialized = 2,

    #[error("Escrow not initialized")]
    NotInitialized = 3,

    #[error("Invalid secret")]
    InvalidSecret = 4,

    #[error("Invalid Merkle proof")]
    InvalidMerkleProof = 5,

    #[error("Timelock not expired")]
    TimelockNotExpired = 6,

    #[error("Unauthorized")]
    Unauthorized = 7,

    #[error("Arithmetic overflow")]
    Overflow = 8,

    #[error("Invalid account")]
    InvalidAccount = 9,

    #[error("Insufficient funds")]
    InsufficientFunds = 10,

    #[error("Already withdrawn")]
    AlreadyWithdrawn = 11,

    #[error("Invalid filled index")]
    InvalidFilledIndex = 12,

    #[error("Merkle proof too deep")]
    MerkleProofTooDeep = 13,

    #[error("Invalid token mint")]
    InvalidTokenMint = 14,

    #[error("Safety deposit mismatch")]
    SafetyDepositMismatch = 15,
}

impl PrintProgramError for EscrowError {
    fn print<E>(&self) {
        match self {
            EscrowError::InvalidInstruction => solana_program::msg!("Error: Invalid instruction"),
            EscrowError::InvalidPDA => solana_program::msg!("Error: Invalid PDA"),
            EscrowError::AlreadyInitialized => {
                solana_program::msg!("Error: Escrow already initialized")
            }
            EscrowError::NotInitialized => solana_program::msg!("Error: Escrow not initialized"),
            EscrowError::InvalidSecret => solana_program::msg!("Error: Invalid secret"),
            EscrowError::InvalidMerkleProof => solana_program::msg!("Error: Invalid Merkle proof"),
            EscrowError::TimelockNotExpired => solana_program::msg!("Error: Timelock not expired"),
            EscrowError::Unauthorized => solana_program::msg!("Error: Unauthorized"),
            EscrowError::Overflow => solana_program::msg!("Error: Arithmetic overflow"),
            EscrowError::InvalidAccount => solana_program::msg!("Error: Invalid account"),
            EscrowError::InsufficientFunds => solana_program::msg!("Error: Insufficient funds"),
            EscrowError::AlreadyWithdrawn => solana_program::msg!("Error: Already withdrawn"),
            EscrowError::InvalidFilledIndex => solana_program::msg!("Error: Invalid filled index"),
            EscrowError::MerkleProofTooDeep => solana_program::msg!("Error: Merkle proof too deep"),
            EscrowError::InvalidTokenMint => solana_program::msg!("Error: Invalid token mint"),
            EscrowError::SafetyDepositMismatch => {
                solana_program::msg!("Error: Safety deposit mismatch")
            }
        }
    }
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for EscrowError {
    fn type_of() -> &'static str {
        "EscrowError"
    }
}
