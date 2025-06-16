use num_derive::FromPrimitive;
use pinocchio::program_error::ProgramError;
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

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        // Log the error when converting
        match &e {
            EscrowError::InvalidInstruction => pinocchio::log::info!("Error: Invalid instruction"),
            EscrowError::InvalidPDA => pinocchio::log::info!("Error: Invalid PDA"),
            EscrowError::AlreadyInitialized => {
                pinocchio::log::info!("Error: Escrow already initialized")
            }
            EscrowError::NotInitialized => pinocchio::log::info!("Error: Escrow not initialized"),
            EscrowError::InvalidSecret => pinocchio::log::info!("Error: Invalid secret"),
            EscrowError::InvalidMerkleProof => pinocchio::log::info!("Error: Invalid Merkle proof"),
            EscrowError::TimelockNotExpired => pinocchio::log::info!("Error: Timelock not expired"),
            EscrowError::Unauthorized => pinocchio::log::info!("Error: Unauthorized"),
            EscrowError::Overflow => pinocchio::log::info!("Error: Arithmetic overflow"),
            EscrowError::InvalidAccount => pinocchio::log::info!("Error: Invalid account"),
            EscrowError::InsufficientFunds => pinocchio::log::info!("Error: Insufficient funds"),
            EscrowError::AlreadyWithdrawn => pinocchio::log::info!("Error: Already withdrawn"),
            EscrowError::InvalidFilledIndex => pinocchio::log::info!("Error: Invalid filled index"),
            EscrowError::MerkleProofTooDeep => pinocchio::log::info!("Error: Merkle proof too deep"),
            EscrowError::InvalidTokenMint => pinocchio::log::info!("Error: Invalid token mint"),
            EscrowError::SafetyDepositMismatch => {
                pinocchio::log::info!("Error: Safety deposit mismatch")
            }
        }
        ProgramError::Custom(e as u32)
    }
}