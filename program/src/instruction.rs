use pinocchio::pubkey::Pubkey;
use solana_program::program_error::ProgramError;

/// Escrow initialization parameters
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EscrowInit {
    /// EVM maker address on Solana (just opaque bytes)
    pub maker: Pubkey,
    /// The resolver signer
    pub resolver: Pubkey,
    /// Keccak-256 digest
    pub hash_secret: [u8; 32],
    /// SPL token mint
    pub token_mint: Pubkey,
    /// Token amount
    pub amount: u64,
    /// Native SOL deposit
    pub safety_deposit_lamports: u64,
    /// 0 for single-fill swaps
    pub merkle_root: [u8; 32],
    /// Seconds offsets
    pub timelocks: [u64; 7],
    /// PDA bump
    pub bump: u8,
}

/// Escrow instruction enum
#[derive(Clone, Debug)]
pub enum EscrowInstruction {
    /// Create the destination escrow PDA
    /// Accounts: [payer, resolver, escrow_pda, system, token, clock]
    CreateDstEscrow(EscrowInit),

    /// Withdraw to caller with secret + optional Merkle proof
    Withdraw {
        secret: [u8; 32],
        proof: Vec<[u8; 32]>,
    },

    /// Withdraw to target with secret + optional Merkle proof
    WithdrawTo {
        secret: [u8; 32],
        target: Pubkey,
        proof: Vec<[u8; 32]>,
    },

    /// Cancel after timelock
    Cancel,

    /// Public cancel after timelock
    PublicCancel,

    /// Execute withdraw via public path
    PublicWithdraw {
        secret: [u8; 32],
        proof: Vec<[u8; 32]>,
    },

    /// Drain dust after long timeout
    RescueFunds,
}

impl EscrowInstruction {
    /// Unpacks instruction data
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let discriminator = input[0];
        let data = &input[1..];

        match discriminator {
            0 => {
                // CreateDstEscrow
                if data.len() < std::mem::size_of::<EscrowInit>() {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let init = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const EscrowInit) };

                Ok(EscrowInstruction::CreateDstEscrow(init))
            }
            1 => {
                // Withdraw
                if data.len() < 32 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let secret = <[u8; 32]>::try_from(&data[0..32])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                let proof = Self::unpack_proof(&data[32..])?;

                Ok(EscrowInstruction::Withdraw { secret, proof })
            }
            2 => {
                // WithdrawTo
                if data.len() < 64 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let secret = <[u8; 32]>::try_from(&data[0..32])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                let target = Pubkey::from(
                    <[u8; 32]>::try_from(&data[32..64])
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                let proof = Self::unpack_proof(&data[64..])?;

                Ok(EscrowInstruction::WithdrawTo {
                    secret,
                    target,
                    proof,
                })
            }
            3 => Ok(EscrowInstruction::Cancel),
            4 => Ok(EscrowInstruction::PublicCancel),
            5 => {
                // PublicWithdraw
                if data.len() < 32 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let secret = <[u8; 32]>::try_from(&data[0..32])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                let proof = Self::unpack_proof(&data[32..])?;

                Ok(EscrowInstruction::PublicWithdraw { secret, proof })
            }
            6 => Ok(EscrowInstruction::RescueFunds),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    /// Helper to unpack Merkle proof
    fn unpack_proof(data: &[u8]) -> Result<Vec<[u8; 32]>, ProgramError> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        if data.len() % 32 != 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let proof_len = data.len() / 32;
        if proof_len > 32 {
            // Maximum Merkle depth is 32
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut proof = Vec::with_capacity(proof_len);
        for i in 0..proof_len {
            let start = i * 32;
            let end = start + 32;
            let hash = <[u8; 32]>::try_from(&data[start..end])
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            proof.push(hash);
        }

        Ok(proof)
    }
}
