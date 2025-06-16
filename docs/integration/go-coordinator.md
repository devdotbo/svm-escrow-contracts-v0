# Integration Guide - 1inch Fusion SVM Escrow

## Overview

This document provides integration instructions for the 1inch Fusion cross-chain atomic swap system, specifically for integrating the Solana (SVM) escrow program with the Go coordinator and EVM source chain contracts.

## Architecture

### Cross-Chain Flow

1. **Source Chain (EVM)**:
   - User creates source escrow with tokens
   - Resolver monitors and prepares destination escrow

2. **Destination Chain (Solana)**:
   - Resolver creates matching destination escrow
   - User reveals secret to claim tokens
   - Resolver claims source tokens with same secret

3. **Coordinator (Go)**:
   - Monitors both chains
   - Manages PDA derivation
   - Coordinates timelock transitions

## PDA Derivation

### Deterministic Address Generation

The escrow PDA must be derived identically in both the on-chain program and off-chain coordinator:

```go
// Go implementation
func DeriveEscrowPDA(
    programID solana.PublicKey,
    maker solana.PublicKey,
    resolver solana.PublicKey,
    hashSecret [32]byte,
) (pda solana.PublicKey, bump uint8) {
    seeds := [][]byte{
        []byte("escrow"),
        maker.Bytes(),
        resolver.Bytes(),
        hashSecret[:],
        {0x00}, // ROLE_BYTE_DST
    }
    return solana.FindProgramAddress(seeds, programID)
}
```

### Important Notes
- Seeds must be in exact order: `["escrow", maker, resolver, hash_secret, role_byte]`
- Role byte is `0x00` for destination escrows on Solana
- All addresses are 32-byte Pubkeys
- Hash secret is keccak256 output from EVM

## Instruction Data Formats

### CreateDstEscrow

```rust
// Discriminator: 0x00
struct CreateDstEscrow {
    maker: [u8; 32],                  // EVM address as Pubkey
    resolver: [u8; 32],                // Resolver's Solana pubkey
    hash_secret: [u8; 32],             // keccak256(secret)
    token_mint: [u8; 32],              // SPL token mint
    amount: u64,                       // Token amount (little-endian)
    safety_deposit_lamports: u64,      // Safety deposit (little-endian)
    merkle_root: [u8; 32],             // 0 for single-fill
    timelocks: [u64; 7],               // Timelock array (little-endian)
    bump: u8,                          // PDA bump seed
}
```

Total size: 1 + 293 = 294 bytes

### Withdraw

```rust
// Discriminator: 0x01
struct Withdraw {
    secret: [u8; 32],                  // Pre-image of hash_secret
    proof: Vec<[u8; 32]>,              // Merkle proof (0-32 siblings)
}
```

### Account Order

All instructions expect accounts in specific order:

**CreateDstEscrow**:
1. `[WRITE, SIGNER]` Payer
2. `[SIGNER]` Resolver
3. `[WRITE]` Escrow PDA
4. `[]` System Program
5. `[]` Token Program
6. `[]` Clock Sysvar

**Withdraw/WithdrawTo**:
1. `[WRITE]` Escrow PDA
2. `[WRITE]` Maker token account
3. `[WRITE]` Escrow token account
4. `[WRITE]` Caller (for safety deposit)
5. `[]` Token Program
6. `[]` Clock Sysvar
7. `[]` System Program
8. `[WRITE, optional]` Target account (WithdrawTo only)

## Timelock Management

### Timelock Indices

| Index | Stage | Description |
|-------|-------|-------------|
| 0 | src_exclusive_withdraw | Source chain exclusive (passed) |
| 1 | src_public_withdraw | Source chain public (passed) |
| 2 | dst_exclusive_withdraw | Resolver-only withdraw |
| 3 | dst_public_withdraw | Anyone can withdraw |
| 4 | dst_exclusive_cancel | Resolver-only cancel |
| 5 | dst_public_cancel | Anyone can cancel |
| 6 | rescue | Dust cleanup |

### Timing Coordination

```go
// Example timelock calculation
baseTime := time.Now().Unix()
timelocks := [7]uint64{
    0,     // Already passed
    0,     // Already passed
    300,   // +5 minutes
    600,   // +10 minutes
    900,   // +15 minutes
    1200,  // +20 minutes
    86400, // +24 hours
}
```

## Error Handling

### Program Errors

| Error | Code | Description |
|-------|------|-------------|
| InvalidSecret | 4 | Secret doesn't match hash |
| InvalidMerkleProof | 5 | Merkle proof verification failed |
| TimelockNotExpired | 6 | Operation attempted too early |
| Unauthorized | 7 | Caller not authorized |

### Recovery Procedures

1. **Failed Withdraw**: Check secret and timing
2. **Failed Cancel**: Verify timelock expiration
3. **Stuck Funds**: Use RescueFunds after 24h

## Testing

### Devnet Deployment

```bash
# Deploy program
solana program deploy program/target/deploy/svm_escrow.so

# Note the program ID
export PROGRAM_ID=<deployed-program-id>
```

### Integration Test

```go
// Create test escrow
func TestCreateEscrow(t *testing.T) {
    // 1. Generate test keys
    maker := solana.NewWallet()
    resolver := solana.NewWallet()
    secret := []byte("test_secret_32_bytes_padding____")
    hashSecret := keccak256(secret)
    
    // 2. Derive PDA
    pda, bump := DeriveEscrowPDA(
        programID,
        maker.PublicKey(),
        resolver.PublicKey(),
        hashSecret,
    )
    
    // 3. Create instruction
    // ... (pack instruction data)
    
    // 4. Send transaction
    // ... (submit to chain)
}
```

## Performance Considerations

### Compute Units

- CreateDstEscrow: ~50k CU
- Withdraw (no Merkle): ~80k CU
- Withdraw (32-level Merkle): ~180k CU
- Cancel: ~60k CU

### Optimization Tips

1. Pre-compute PDA addresses off-chain
2. Batch multiple operations when possible
3. Use getProgramAccounts sparingly
4. Cache account data when monitoring

## Security Checklist

- [ ] Verify PDA derivation matches exactly
- [ ] Check all timelock boundaries
- [ ] Validate secret format (32 bytes)
- [ ] Ensure SPL token amounts are correct
- [ ] Monitor for timelock expiration
- [ ] Handle all error cases
- [ ] Test with mainnet-beta parameters

## Example Integration

See `tests/e2e_integration.go` for a complete example of:
- Cross-chain escrow creation
- Secret revelation flow
- Error recovery procedures
- Monitoring implementation