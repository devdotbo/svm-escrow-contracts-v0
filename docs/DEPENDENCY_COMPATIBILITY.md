# Dependency Compatibility Guide

## Critical Version Alignment

When upgrading Solana dependencies, **ALL** Solana crates must use the same version to avoid conflicts:

```toml
[dependencies]
# Program dependencies (if needed)
solana-program = "2.3.0"  # Only if pinocchio can't replace it

[dev-dependencies]
# Test dependencies - MUST match
solana-program-test = "2.3.0"
solana-sdk = "2.3.0"
solana-program = "2.3.0"  # May be needed for test utilities
```

## Why Version Alignment Matters

### 1. **Type Conflicts**
Different versions export different types:
- `Pubkey` from 2.1 ≠ `Pubkey` from 2.3
- Causes "expected struct X, found struct X" errors

### 2. **Feature Gates**
Newer versions may have different feature flags:
- API changes between versions
- Deprecated/removed functionality

### 3. **Dependency Diamond Problem**
```
Your Program
├── solana-program 2.3.0
└── spl-token 6.0
    └── solana-program 2.1.0  ❌ CONFLICT!
```

## SPL Token Compatibility

SPL Token versions are tied to Solana versions:

| SPL Token | Compatible Solana |
|-----------|------------------|
| 6.0       | 2.0-2.1         |
| 7.0       | 2.2-2.3         |
| 8.0       | 2.4+            |

## Pinocchio Integration

### Can Replace
- ✅ Basic types (Pubkey, AccountInfo)
- ✅ Syscalls (keccak256, logging)
- ✅ Program entrypoint
- ✅ CPI functionality

### Cannot Replace (Need solana-program)
- ❌ Complex sysvars access
- ❌ Some error conversions
- ❌ Test framework integration

## Upgrade Strategy

### Option 1: Full Pinocchio (Recommended)
```toml
[dependencies]
pinocchio = "0.8.4"
# No solana-program!

[dev-dependencies]
solana-program-test = "2.3.0"
solana-sdk = "2.3.0"
```

### Option 2: Minimal solana-program
```toml
[dependencies]
pinocchio = "0.8.4"
solana-program = { version = "2.3.0", default-features = false }

[dev-dependencies]
solana-program-test = "2.3.0"
solana-sdk = "2.3.0"
```

## Testing Compatibility

### Keccak256 in Tests
Since tests use solana-program-test, we can use solana-program's keccak:

```rust
// In tests only
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak;
    keccak::hashv(&[data]).to_bytes()
}
```

### Program Uses Pinocchio
```rust
// In program
fn verify_secret(secret: &[u8; 32], expected: &[u8; 32]) -> ProgramResult {
    let computed = pinocchio::syscalls::sol_keccak256(secret);
    if computed != *expected {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}
```

## Common Issues & Solutions

### Issue: "multiple versions of crate X"
**Solution**: Ensure all Solana deps use same version

### Issue: "trait X is not implemented"
**Solution**: Check feature flags match across deps

### Issue: "cannot find type X in crate Y"
**Solution**: API may have changed, check migration guide

### Issue: Build succeeds but tests fail
**Solution**: Test/program dependency mismatch

## Version Lock File

Create `.cargo/config.toml`:
```toml
[patch.crates-io]
# Force all deps to use same version
solana-program = { version = "=2.3.0" }
solana-sdk = { version = "=2.3.0" }
```

## Verification Commands

```bash
# Check dependency tree
cargo tree | grep solana

# Verify no duplicates
cargo tree -d | grep solana

# Clean build
cargo clean && cargo build-sbf
```