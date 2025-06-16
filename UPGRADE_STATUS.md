# Upgrade Status

## Current State
- **Branch**: upgrade-rust-latest
- **Phase**: Planning complete, ready to execute
- **Target**: Rust 1.87.0, Edition 2024, Pinocchio 0.8.4

## Upgrade Checklist

### Phase 1: Toolchain Update
- [ ] Update rust-toolchain.toml to 1.87.0
- [ ] Update all Cargo.toml files to edition = "2024"

### Phase 2: Dependencies
- [ ] Update workspace Cargo.toml:
  ```toml
  pinocchio = "0.8.4"
  solana-program = "=2.2.16"
  solana-sdk = "=2.2.16"
  solana-program-test = "=2.2.16"
  spl-token = "6.1"  # Compatible with 2.2.x
  ```

### Phase 3: Code Migration
- [ ] Replace `solana_program::keccak::hashv` → `pinocchio::syscalls::sol_keccak256`
- [ ] Replace `solana_program::msg!` → `pinocchio::log::info!`
- [ ] Remove solana-program from main program dependencies
- [ ] Update any remaining solana-program imports

### Phase 4: Testing
- [ ] Fix any compilation errors
- [ ] Run all tests
- [ ] Verify program builds with `cargo build-sbf`

## Migration Notes

### Keccak256 Changes
```rust
// Old (processor.rs lines 222, 357)
use solana_program::keccak;
let hash = keccak::hashv(&[data]).to_bytes();

// New
use pinocchio::syscalls::sol_keccak256;
let hash = sol_keccak256(data);
```

### Logging Changes
```rust
// Old (processor.rs lines 157, 310, 469, 592)
solana_program::msg!("Message");

// New
pinocchio::log::info!("Message");
```

## Next Step
Execute Phase 1: Update toolchain configuration