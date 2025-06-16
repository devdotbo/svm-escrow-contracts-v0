# CLAUDE.md - Essential Implementation Reference

## Critical Constraints
- **Framework**: pinocchio (NO Anchor)
- **Rust**: 1.87.0 target
- **Program**: Immutable (no upgrade authority)

## Commands
```bash
# Build
cargo build-sbf --manifest-path program/Cargo.toml -- --release

# Test
cargo test -- --nocapture

# Lint/Format
cargo fmt -- --check
cargo clippy -- -D warnings
```

## PDA Seeds (EXACT)
```
["escrow", maker, resolver, hash_secret, role_byte]
role_byte = 0x00 for Dst escrow on Solana
```

## Core Instructions
1. `CreateDstEscrow` - Init PDA with safety deposit
2. `Withdraw/WithdrawTo` - Secret + optional Merkle proof
3. `Cancel/PublicCancel` - After timelock
4. `PublicWithdraw` - Public phase withdrawal
5. `RescueFunds` - Dust cleanup

## Timelock Indices
```
[2] dst_exclusive_withdraw - Only resolver
[3] dst_public_withdraw    - Anyone
[4] dst_exclusive_cancel   - Only resolver  
[5] dst_public_cancel      - Anyone
[6] rescue                 - Anyone (dust)
```

## Security Musts
- Keccak256 for secret verification
- Merkle depth ≤ 32
- Check overflow with checked_add/mul
- Transfer lamports LAST (re-entrancy)
- Keep PDA rent-exempt always
- Close PDA after last action

## Testing Gates
- `happy_path.rs` - Basic flow
- `public_withdraw.rs` - Public phase
- `cancel.rs` - Cancel flow
- `merkle_depth32.rs` - Deep proofs
- `cu_budget.rs` - Performance benchmarks