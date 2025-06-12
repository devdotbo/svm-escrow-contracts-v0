# 1inch Fusion SVM Escrow Contracts

Solana (SVM) implementation of the destination-chain escrow for 1inch Fusion+ cross-chain atomic swaps.

## Overview

This program implements a trustless escrow system on Solana that works in tandem with EVM-based source chain escrows. It enables atomic cross-chain swaps with time-based security guarantees and optional Merkle tree support for batch fills.

### Key Features
- **Cross-chain atomic swaps** between EVM and Solana
- **Time-locked escrows** with phased withdrawal/cancel periods
- **Merkle tree support** for efficient batch processing (up to 32 levels)
- **Safety deposits** to incentivize proper resolution
- **Gas-optimized** implementation using pinocchio framework

## Architecture

### Program Structure
```
fusion-svm/
├── program/           # On-chain program (pinocchio-based)
├── tests/            # Integration tests
├── docs/             # Additional documentation
└── .github/          # CI/CD workflows
```

### Core Components

1. **Escrow PDA** - Stores swap details and enforces time-based logic
2. **Instruction Handlers** - Process create, withdraw, cancel operations
3. **Merkle Verifier** - Validates proofs for batch fills
4. **Timelock System** - 7-stage security model matching EVM side

## Setup

### Prerequisites
- Rust 1.77.0 or later
- Solana CLI tools
- cargo-build-bpf

### Installation
```bash
# Clone the repository
git clone <repo-url>
cd svm-escrow-contracts

# Install dependencies
cargo build

# Build BPF program
cargo build-bpf --manifest-path program/Cargo.toml
```

### Testing
```bash
# Run all tests
cargo test -- --nocapture

# Run specific test suite
cargo test happy_path -- --nocapture

# Fuzz testing
cd program && cargo fuzz run merkle_proof -- -runs=10000
```

## Usage

### Creating an Escrow
The resolver creates a destination escrow with matching parameters from the source chain:
- Maker address (from EVM)
- Secret hash (keccak256)
- Token mint and amount
- Timelock schedule
- Optional Merkle root for batch fills

### Withdrawing Funds
1. **Exclusive Phase**: Only resolver can withdraw (timelock[2])
2. **Public Phase**: Anyone can withdraw with valid secret (timelock[3])

### Cancellation
Similar phased approach for cancellations (timelock[4] and timelock[5])

## Security Model

The program implements multiple layers of security:
- **Time-based phases** prevent front-running and ensure fairness
- **Merkle proofs** prevent double-spending in batch scenarios
- **Safety deposits** incentivize proper behavior
- **Rent-exemption** ensures PDA persistence
- **Overflow protection** on all arithmetic operations

## Integration

See `docs/integration.md` for detailed integration instructions with the Go coordinator and EVM contracts.

## License

[License details here]