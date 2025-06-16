use crate::state::Escrow;
use pinocchio::pubkey::Pubkey;

#[test]
fn test_escrow_size() {
    // Verify the size calculation is correct
    let expected_size = 32 + 32 + 32 + 32 + 8 + 8 + 32 + 4 + (8 * 7) + 8 + 1;
    assert_eq!(Escrow::LEN, expected_size);
    assert_eq!(Escrow::LEN, 293);
}

#[test]
fn test_pda_derivation() {
    let program_id = Pubkey::new_unique();
    let maker = Pubkey::new_unique();
    let resolver = Pubkey::new_unique();
    let hash_secret = [1u8; 32];

    let (pda1, bump1) = Escrow::derive_pda(&maker, &resolver, &hash_secret, &program_id);
    let (pda2, bump2) = Escrow::derive_pda(&maker, &resolver, &hash_secret, &program_id);

    // Should be deterministic
    assert_eq!(pda1, pda2);
    assert_eq!(bump1, bump2);
}

#[test]
fn test_pda_uniqueness() {
    let program_id = Pubkey::new_unique();
    let maker = Pubkey::new_unique();
    let resolver = Pubkey::new_unique();
    let hash_secret1 = [1u8; 32];
    let hash_secret2 = [2u8; 32];

    let (pda1, _) = Escrow::derive_pda(&maker, &resolver, &hash_secret1, &program_id);
    let (pda2, _) = Escrow::derive_pda(&maker, &resolver, &hash_secret2, &program_id);

    // Different secrets should produce different PDAs
    assert_ne!(pda1, pda2);
}

#[test]
fn test_escrow_constants() {
    assert_eq!(Escrow::SEED_PREFIX, b"escrow");
    assert_eq!(Escrow::ROLE_BYTE_DST, 0x00);

    // Timelock indices
    assert_eq!(Escrow::TL_SRC_EXCLUSIVE_WITHDRAW, 0);
    assert_eq!(Escrow::TL_SRC_PUBLIC_WITHDRAW, 1);
    assert_eq!(Escrow::TL_DST_EXCLUSIVE_WITHDRAW, 2);
    assert_eq!(Escrow::TL_DST_PUBLIC_WITHDRAW, 3);
    assert_eq!(Escrow::TL_DST_EXCLUSIVE_CANCEL, 4);
    assert_eq!(Escrow::TL_DST_PUBLIC_CANCEL, 5);
    assert_eq!(Escrow::TL_RESCUE, 6);
}
