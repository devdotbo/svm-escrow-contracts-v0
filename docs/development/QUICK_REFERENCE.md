# Quick Reference

## Essential Commands

```bash
# Build
cargo build-sbf --manifest-path program/Cargo.toml -- --release

# Test
cargo test -- --nocapture

# Lint/Format
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Project Constraints
- **Framework**: pinocchio 0.8.4 (NO Anchor)
- **Rust**: 1.87.0
- **Edition**: 2024
- **Solana**: =2.2.16 (exact pin)
- **Program**: Immutable (no upgrade authority)

## PDA Seeds (EXACT)
```
["escrow", maker, resolver, hash_secret, role_byte]
role_byte = 0x00 for Dst escrow on Solana
```

## Timelock Indices
```
[2] dst_exclusive_withdraw - Only resolver
[3] dst_public_withdraw    - Anyone
[4] dst_exclusive_cancel   - Only resolver  
[5] dst_public_cancel      - Anyone
[6] rescue                 - Anyone (dust)
```

## Security Checklist
- ✓ Keccak256 for secret verification
- ✓ Merkle depth ≤ 32
- ✓ Check overflow with checked_add/mul
- ✓ Transfer lamports LAST (re-entrancy)
- ✓ Keep PDA rent-exempt always
- ✓ Close PDA after last action