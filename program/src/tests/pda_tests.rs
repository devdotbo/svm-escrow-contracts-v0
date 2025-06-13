use crate::state::Escrow;
use pinocchio::pubkey::Pubkey;

#[test]
fn test_pda_seed_construction() {
    let maker = Pubkey::new_unique();
    let resolver = Pubkey::new_unique();
    let hash_secret = [42u8; 32];
    
    // Verify seeds are constructed correctly
    let seeds = [
        Escrow::SEED_PREFIX,
        maker.as_ref(),
        resolver.as_ref(),
        &hash_secret,
        &[Escrow::ROLE_BYTE_DST],
    ];
    
    assert_eq!(seeds[0], b"escrow");
    assert_eq!(seeds[1].len(), 32);
    assert_eq!(seeds[2].len(), 32);
    assert_eq!(seeds[3].len(), 32);
    assert_eq!(seeds[4], &[0x00]);
}

#[test]
fn test_pda_with_different_roles() {
    let program_id = Pubkey::new_unique();
    let maker = Pubkey::new_unique();
    let resolver = Pubkey::new_unique();
    let hash_secret = [1u8; 32];
    
    // Our implementation uses ROLE_BYTE_DST (0x00)
    let (pda_dst, _) = Escrow::derive_pda(&maker, &resolver, &hash_secret, &program_id);
    
    // If we had a different role byte, it would produce a different PDA
    let (pda_alt, _) = Pubkey::find_program_address(
        &[
            Escrow::SEED_PREFIX,
            maker.as_ref(),
            resolver.as_ref(),
            &hash_secret,
            &[0x01], // Different role byte
        ],
        &program_id,
    );
    
    assert_ne!(pda_dst, pda_alt);
}