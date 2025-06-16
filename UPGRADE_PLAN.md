# Rust Latest Version Upgrade Plan

## Overview
Upgrade the SVM escrow contracts project to:
- Rust 1.87.0 (from 1.77.0)
- Rust Edition 2024 (from 2021)
- Pinocchio 0.8.4 (from 0.5)
- Latest dependency versions
- Minimize/eliminate solana-program usage

## Phase 1: Dependency Analysis

### Current Dependencies
- pinocchio: 0.5 → 0.8.4
- solana-program: 2.1 → 2.2.16 (only if absolutely necessary)
- solana-sdk: 2.1 → 2.2.16 (only in tests)
- solana-program-test: 2.1 → 2.2.16 (tests only)
- spl-token: 6.0 → latest compatible with 2.2.x
- thiserror: 1.0 → latest
- num-derive: 0.4 → latest
- num-traits: 0.2 → latest
- bincode: 1.3 → latest
- tokio: 1.x → latest

### Compatibility Notes
- **Version Alignment**: solana-program, solana-sdk, and solana-program-test MUST use the same version (2.2.16)
- **Production Ready**: v2.2.16 is the latest stable release for Mainnet Beta (v2.3.0 is testnet only)
- **SPL Token**: May need specific version compatible with solana-program 2.2.x
- **Rust Version**: Our target 1.87.0 is compatible
- **Ubuntu**: Requires Ubuntu 22.04 or later (20.04 is EOL)

### solana-program Usage Analysis
Currently using solana-program for:
1. **Keccak256 hashing** - Can replace with pinocchio's `sol_keccak256`
2. **msg! macro** - Can replace with pinocchio's `log::info!`
3. **Error types** - Can use pinocchio's program_error
4. **Account info** - Already using pinocchio
5. **Entrypoint** - Already using pinocchio

## Phase 2: Toolchain Update

### Tasks:
1. Update rust-toolchain.toml to 1.87.0
2. Update edition in Cargo.toml to "2024"
3. Update resolver to "2" (already done)

## Phase 3: Pinocchio Migration

### Replace solana-program usage:
1. **Keccak256 hashing**
   - From: `solana_program::keccak::hashv`
   - To: `pinocchio::syscalls::sol_keccak256`
   
2. **Logging**
   - From: `solana_program::msg!`
   - To: `pinocchio::log::info!`
   
3. **Error handling**
   - Verify pinocchio::program_error compatibility
   - Update custom error mappings if needed

4. **Remove solana-program dependency**
   - Only keep in tests where needed for test framework

## Phase 4: Code Updates

### Files to modify:
1. **processor.rs**
   - Replace keccak imports/usage (lines 222, 357)
   - Replace msg! macro calls (lines 157, 310, 469, 592)
   - Update any other solana-program imports

2. **state.rs**
   - Verify PDA derivation still works with pinocchio
   - Check account info handling

3. **instruction.rs**
   - Update any solana-program imports
   - Verify instruction parsing compatibility

4. **error.rs**
   - Ensure error conversions work with pinocchio

5. **lib.rs**
   - Verify entrypoint configuration

## Phase 5: Test Infrastructure

### Keep solana-sdk/solana-program-test in tests only:
- Required for test framework (ProgramTest)
- Update to version 2.3.0
- Keep isolated to test dependencies

### Version Compatibility Critical:
```toml
[dev-dependencies]
# These MUST all be the same version
solana-program-test = "2.2.16"
solana-sdk = "2.2.16"
# If we need solana-program for keccak256 fallback
solana-program = "2.2.16"
```

### Update test utilities:
- Ensure compatibility with new versions
- Fix any breaking changes in test setup
- Verify keccak256 helper function works

## Phase 6: Build and Verification

### Steps:
1. Run `cargo update` to get latest compatible versions
2. Build with `cargo build-sbf`
3. Verify BPF size still < 120 KiB
4. Run all tests
5. Check CU budget benchmarks

## Breaking Changes to Watch

### Rust 1.87.0:
- New lifetime elision rules
- Potential const evaluation changes
- Updated trait implementations

### Pinocchio 0.8.4:
- API changes from 0.5
- New module organization
- Syscall interface updates

### Edition 2024:
- New language features
- Potential syntax changes
- Updated prelude items

## Risk Mitigation

1. **Create upgrade branch** (done)
2. **Incremental updates** - One component at a time
3. **Comprehensive testing** after each change
4. **BPF size monitoring** - Ensure no regressions
5. **CU budget verification** - Maintain < 200k target

## Implementation Order

1. Update toolchain and edition
2. Update workspace dependencies
3. Migrate pinocchio usage in program
4. Update test dependencies
5. Fix any compilation issues
6. Run full test suite
7. Verify BPF size and CU usage
8. Document any API changes

## Success Criteria

- [ ] Builds with Rust 1.87.0
- [ ] Uses Rust Edition 2024
- [ ] Pinocchio 0.8.4 integrated
- [ ] No solana-program dependency in main program
- [ ] All tests pass
- [ ] BPF size < 120 KiB maintained
- [ ] CU budget < 200k maintained
- [ ] No functional regressions