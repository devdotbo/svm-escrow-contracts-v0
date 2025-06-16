#![cfg_attr(not(feature = "no-entrypoint"), no_std)]
#![cfg_attr(not(feature = "no-entrypoint"), no_main)]

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(test)]
mod tests;

// Export types for use by clients
pub use error::EscrowError;
pub use instruction::{EscrowInit, EscrowInstruction};
pub use state::Escrow;
