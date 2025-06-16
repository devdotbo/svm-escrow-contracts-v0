# Implementation Plan - 1inch Fusion SVM Escrow

## Week 1: Project Scaffold & Core Types

### Tasks
- [ ] Initialize Cargo workspace structure
- [ ] Set up program/Cargo.toml with pinocchio dependency
- [ ] Create Xargo.toml for BPF compilation
- [ ] Implement `state.rs` with Escrow zero-copy struct
- [ ] Generate `instruction.rs` enum using pinocchio codegen
- [ ] Set up basic error types in `error.rs`
- [ ] Create lib.rs with entrypoint skeleton
- [ ] Configure GitHub Actions CI workflow

### Acceptance Criteria
- `cargo check` passes
- CI pipeline runs on push/PR
- Escrow struct matches spec exactly (field order, types)
- BPF build succeeds (even if empty)

### Implementation Notes
```rust
// Escrow struct must be #[zero_copy] with exact field order:
// maker, resolver, hash_secret, token_mint, amount, 
// safety_deposit_lamports, merkle_root, filled_index,
// timelocks[7], deployed_at, bump
```

---

## Week 2: Create Escrow Implementation

### Tasks
- [ ] Implement `create_dst_escrow` handler in processor.rs
- [ ] Add PDA derivation logic with exact seeds
- [ ] Implement rent-exemption calculation
- [ ] Add safety deposit validation
- [ ] Create unit test framework setup
- [ ] Write `happy_path.rs` test skeleton
- [ ] Add account validation helpers

### Acceptance Criteria
- Create instruction successfully initializes PDA
- PDA address matches expected derivation
- Safety deposit correctly added to rent-exemption
- Unit tests compile (not yet passing)

### Key Validations
```rust
// PDA seeds: ["escrow", maker, resolver, hash_secret, role_byte]
// role_byte = 0x00
// Verify signer == resolver
// Ensure PDA uninitialized before creation
```

---

## Week 3: Withdraw Implementation

### Tasks
- [ ] Implement keccak256 secret verification
- [ ] Add `withdraw` instruction handler
- [ ] Add `withdraw_to` variant
- [ ] Implement SPL token transfer logic
- [ ] Add timelock validation (stages 2 & 3)
- [ ] Handle single-fill vs merkle cases
- [ ] Complete happy_path.rs test

### Acceptance Criteria
- Secret verification uses keccak256
- Timelock checks enforce exclusive/public phases
- SPL tokens transferred correctly
- Safety deposit paid to caller
- happy_path.rs test passes

### CU Optimization
- Use `keccak::hashv(&[&secret])` (260 CU)
- Precompute account keys outside loops

---

## Week 4: Cancel & Timelock Logic

### Tasks
- [ ] Implement full timelock ladder validation
- [ ] Add `cancel` instruction handler
- [ ] Add `public_cancel` variant
- [ ] Implement resolver-only vs public logic
- [ ] Create cancel.rs test file
- [ ] Add timelock boundary tests
- [ ] Implement PDA closing logic

### Acceptance Criteria
- All 7 timelock stages validated correctly
- Cancel returns tokens to resolver
- Safety deposit incentive works
- PDA closed after operations
- cancel.rs test passes

### Security Focus
- Lamports transferred last (re-entrancy)
- PDA remains rent-exempt until closed
- No arithmetic overflows

---

## Week 5: Merkle Proof & CU Optimization

### Tasks
- [ ] Implement Merkle proof verifier (≤32 depth)
- [ ] Add filled_index tracking
- [ ] Optimize hash computations
- [ ] Create merkle_depth32.rs test
- [ ] Create cu_budget.rs benchmarks
- [ ] Profile and optimize hot paths
- [ ] Add batch fill test scenarios

### Acceptance Criteria
- 32-level proofs verify correctly
- filled_index increments properly
- All operations < 200k CU
- No panics on malformed proofs
- Tests pass with measurements

### Optimization Targets
- Merkle verify loop hard-capped at 32
- Account key hashes precomputed
- Minimal heap allocations

---

## Week 6: Public Operations & Fuzzing

### Tasks
- [ ] Implement `public_withdraw` handler
- [ ] Implement `rescue_funds` for dust
- [ ] Create public_withdraw.rs test
- [ ] Set up cargo-fuzz harness
- [ ] Fuzz Merkle proof verifier
- [ ] Fuzz timelock math
- [ ] Add edge case tests

### Acceptance Criteria
- Public operations respect timelocks
- Rescue only after final timeout
- Fuzz finds no panics (10k runs)
- All integration tests green
- Code coverage > 90%

### Fuzz Targets
- Merkle proof validation
- Timelock calculations
- Overflow scenarios

---

## Week 7: Documentation & E2E Integration

### Tasks
- [ ] Write docs/integration.md
- [ ] Document PDA derivation for Go coordinator
- [ ] Create E2E test with mock coordinator
- [ ] Deploy to devnet for testing
- [ ] Verify cross-chain flow
- [ ] Performance benchmarks
- [ ] Final security review

### Acceptance Criteria
- Complete integration guide
- Successful devnet deployment
- E2E swap completes
- All tests pass
- BPF size < 120 KiB
- Documentation complete

### Integration Points
- PDA derivation matches Go coordinator
- Instruction data format documented
- Error codes mapped
- Account order specified

---

## Testing Checklist

### Unit Tests (program/src/tests/)
- [ ] State serialization
- [ ] PDA derivation
- [ ] Timelock calculations
- [ ] Merkle verification

### Integration Tests (tests/)
- [ ] happy_path.rs - Full success flow
- [ ] public_withdraw.rs - Public phase execution
- [ ] cancel.rs - Cancel flows
- [ ] merkle_depth32.rs - Deep proof handling
- [ ] cu_budget.rs - Performance limits

### E2E Tests
- [ ] Cross-chain coordinator integration
- [ ] Devnet deployment verification
- [ ] Multi-fill Merkle scenarios

## Daily Workflow

1. Check CLAUDE.md for constraints
2. Run tests before any commit
3. Verify BPF size after builds
4. Update this plan with progress
5. Document any deviations

## Risk Mitigation

- **BPF Size**: Monitor after each feature
- **CU Budget**: Benchmark continuously  
- **Security**: Fuzz test all user inputs
- **Integration**: Test with coordinator early