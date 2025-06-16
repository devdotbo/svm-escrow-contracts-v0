# 1inch Fusion SVM Escrow Contracts

Solana (SVM) implementation of the destination-chain escrow for 1inch Fusion+ cross-chain atomic swaps.

## Overview

This program implements a trustless escrow system on Solana that works in tandem with EVM-based source chain escrows. It enables atomic cross-chain swaps with time-based security guarantees and optional Merkle tree support for batch fills.

### Key Features
- **Cross-chain atomic swaps** between EVM and Solana
- **Time-locked escrows** with phased withdrawal/cancel periods
- **Merkle tree support** for efficient batch processing (up to 32 levels)
- **Safety deposits** to incentivize proper resolution
- **Zero-copy operations** using pinocchio framework

## Quick Start

```bash
# Build
cargo build-sbf --manifest-path program/Cargo.toml -- --release

# Test
cargo test -- --nocapture

# Check code
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Documentation

- **[Quick Reference](docs/development/QUICK_REFERENCE.md)** - Essential commands and constraints
- **[Upgrade Status](UPGRADE_STATUS.md)** - Current upgrade progress
- **[Upgrade Plan](UPGRADE_PLAN.md)** - Detailed upgrade roadmap
- **[Integration Guide](docs/integration/go-coordinator.md)** - Go coordinator interface
- **[Dependency Guide](docs/development/DEPENDENCY_COMPATIBILITY.md)** - Version alignment

## Project Structure

```
svm-escrow-contracts/
├── program/              # On-chain program (pinocchio)
│   └── src/
│       ├── processor.rs  # Core business logic
│       ├── state.rs      # Escrow data structure
│       └── instruction.rs # Instruction definitions
├── tests/                # Integration tests
├── docs/                 # Documentation
│   ├── development/      # Development guides
│   ├── integration/      # Integration specs
│   └── history/          # Historical docs
└── .github/              # CI/CD workflows
```

## Core Instructions

1. **CreateDstEscrow** - Initialize escrow with safety deposit
2. **Withdraw/WithdrawTo** - Claim with secret + optional Merkle proof
3. **Cancel/PublicCancel** - Refund after timelock
4. **PublicWithdraw** - Public phase withdrawal
5. **RescueFunds** - Cleanup dust after final timeout

## Security Model

- **Time-based phases** prevent front-running
- **Merkle proofs** enable batch processing
- **Safety deposits** incentivize resolution
- **Immutable program** (no upgrade authority)
- **Re-entrancy protection** (transfers last)

## License

MIT