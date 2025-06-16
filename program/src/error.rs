use num_derive::FromPrimitive;
use pinocchio::{msg, program_error::ProgramError};

/// Custom errors for the escrow program
#[derive(Clone, Debug, Eq, FromPrimitive, PartialEq)]
pub enum EscrowError {
    /// Invalid instruction
    InvalidInstruction = 0,

    /// Invalid PDA
    InvalidPDA = 1,

    /// Escrow already initialized
    AlreadyInitialized = 2,

    /// Escrow not initialized
    NotInitialized = 3,

    /// Invalid secret
    InvalidSecret = 4,

    /// Invalid Merkle proof
    InvalidMerkleProof = 5,

    /// Timelock not expired
    TimelockNotExpired = 6,

    /// Unauthorized
    Unauthorized = 7,

    /// Arithmetic overflow
    Overflow = 8,

    /// Invalid account
    InvalidAccount = 9,

    /// Insufficient funds
    InsufficientFunds = 10,

    /// Already withdrawn
    AlreadyWithdrawn = 11,

    /// Invalid filled index
    InvalidFilledIndex = 12,

    /// Merkle proof too deep
    MerkleProofTooDeep = 13,

    /// Invalid token mint
    InvalidTokenMint = 14,

    /// Safety deposit mismatch
    SafetyDepositMismatch = 15,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        // Log the error when converting
        match &e {
            EscrowError::InvalidInstruction => msg!("Error: Invalid instruction"),
            EscrowError::InvalidPDA => msg!("Error: Invalid PDA"),
            EscrowError::AlreadyInitialized => {
                msg!("Error: Escrow already initialized")
            }
            EscrowError::NotInitialized => msg!("Error: Escrow not initialized"),
            EscrowError::InvalidSecret => msg!("Error: Invalid secret"),
            EscrowError::InvalidMerkleProof => msg!("Error: Invalid Merkle proof"),
            EscrowError::TimelockNotExpired => msg!("Error: Timelock not expired"),
            EscrowError::Unauthorized => msg!("Error: Unauthorized"),
            EscrowError::Overflow => msg!("Error: Arithmetic overflow"),
            EscrowError::InvalidAccount => msg!("Error: Invalid account"),
            EscrowError::InsufficientFunds => msg!("Error: Insufficient funds"),
            EscrowError::AlreadyWithdrawn => msg!("Error: Already withdrawn"),
            EscrowError::InvalidFilledIndex => msg!("Error: Invalid filled index"),
            EscrowError::MerkleProofTooDeep => msg!("Error: Merkle proof too deep"),
            EscrowError::InvalidTokenMint => msg!("Error: Invalid token mint"),
            EscrowError::SafetyDepositMismatch => {
                msg!("Error: Safety deposit mismatch")
            }
        }
        ProgramError::Custom(e as u32)
    }
}