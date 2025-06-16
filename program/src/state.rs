use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

/// Zero-copy account structure for the Escrow PDA
/// Must match the exact field order and types from the specification
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Escrow {
    /// Matches EVM's maker address
    pub maker: Pubkey,
    /// Matches EVM's resolver address
    pub resolver: Pubkey,
    /// Keccak256 hash of the secret
    pub hash_secret: [u8; 32],
    /// SPL token mint address
    pub token_mint: Pubkey,
    /// Token amount in smallest unit
    pub amount: u64,
    /// Native SOL deposit paid at init, rewarded at exit
    pub safety_deposit_lamports: u64,
    /// Merkle root for batch fills (0 for single-fill)
    pub merkle_root: [u8; 32],
    /// Next expected Merkle leaf index
    pub filled_index: u32,
    /// Unix-epoch seconds offsets from deployed_at
    pub timelocks: [u64; 7],
    /// Unix-epoch seconds when deployed
    pub deployed_at: u64,
    /// PDA bump seed
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 8 + 8 + 32 + 4 + (8 * 7) + 8 + 1;

    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"escrow";

    /// Role byte for Solana destination escrows
    pub const ROLE_BYTE_DST: u8 = 0x00;

    /// Timelock indices matching TimelocksLib.Stage on EVM
    pub const TL_SRC_EXCLUSIVE_WITHDRAW: usize = 0;
    pub const TL_SRC_PUBLIC_WITHDRAW: usize = 1;
    pub const TL_DST_EXCLUSIVE_WITHDRAW: usize = 2;
    pub const TL_DST_PUBLIC_WITHDRAW: usize = 3;
    pub const TL_DST_EXCLUSIVE_CANCEL: usize = 4;
    pub const TL_DST_PUBLIC_CANCEL: usize = 5;
    pub const TL_RESCUE: usize = 6;

    /// Derives the PDA address for an escrow
    pub fn derive_pda(
        maker: &Pubkey,
        resolver: &Pubkey,
        hash_secret: &[u8; 32],
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                maker.as_ref(),
                resolver.as_ref(),
                hash_secret,
                &[Self::ROLE_BYTE_DST],
            ],
            program_id,
        )
    }

    /// Deserialize from account data
    pub fn from_account_info(account: &AccountInfo) -> Result<&Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let escrow = unsafe { &*(account.data.borrow().as_ptr() as *const Self) };

        Ok(escrow)
    }

    /// Deserialize mutably from account data
    pub fn from_account_info_mut(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account.data_len() < Self::LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let escrow = unsafe { &mut *(account.data.borrow_mut().as_mut_ptr() as *mut Self) };

        Ok(escrow)
    }
}

use solana_program::program_error::ProgramError;
