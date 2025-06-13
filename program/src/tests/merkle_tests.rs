use crate::processor::{hash_pair, verify_merkle_proof};
use crate::error::EscrowError;
use solana_program::program_error::ProgramError;

#[test]
fn test_hash_pair() {
    let left = [1u8; 32];
    let right = [2u8; 32];
    
    let hash1 = hash_pair(&left, &right);
    let hash2 = hash_pair(&left, &right);
    
    // Should be deterministic
    assert_eq!(hash1, hash2);
    
    // Different order should produce different hash
    let hash3 = hash_pair(&right, &left);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_merkle_proof_single_level() {
    // Leaf nodes
    let leaf1 = [1u8; 32];
    let leaf2 = [2u8; 32];
    
    // Root is hash of two leaves
    let root = hash_pair(&leaf1, &leaf2);
    
    // Proof for leaf1 (index 0)
    let proof = vec![leaf2];
    let result = verify_merkle_proof(&leaf1, &proof, &root, 0);
    assert!(result.is_ok());
    
    // Proof for leaf2 (index 1)
    let proof = vec![leaf1];
    let result = verify_merkle_proof(&leaf2, &proof, &root, 1);
    assert!(result.is_ok());
}

#[test]
fn test_merkle_proof_invalid() {
    let leaf = [1u8; 32];
    let wrong_root = [99u8; 32];
    let proof = vec![[2u8; 32]];
    
    let result = verify_merkle_proof(&leaf, &proof, &wrong_root, 0);
    assert!(matches!(
        result,
        Err(ProgramError::Custom(x)) if x == EscrowError::InvalidMerkleProof as u32
    ));
}

#[test]
fn test_merkle_proof_too_deep() {
    let leaf = [1u8; 32];
    let root = [2u8; 32];
    let proof: Vec<[u8; 32]> = (0..33).map(|i| [i as u8; 32]).collect();
    
    let result = verify_merkle_proof(&leaf, &proof, &root, 0);
    assert!(matches!(
        result,
        Err(ProgramError::Custom(x)) if x == EscrowError::MerkleProofTooDeep as u32
    ));
}

#[test]
fn test_merkle_proof_max_depth() {
    // Test with exactly 32 levels (maximum allowed)
    let leaf = [1u8; 32];
    let mut current = leaf;
    let mut proof = Vec::new();
    
    // Build a proof path of depth 32
    for i in 0..32 {
        let sibling = [(i + 2) as u8; 32];
        proof.push(sibling);
        current = hash_pair(&current, &sibling);
    }
    
    let root = current;
    
    // Should succeed with exactly 32 levels
    let result = verify_merkle_proof(&leaf, &proof, &root, 0);
    assert!(result.is_ok());
}