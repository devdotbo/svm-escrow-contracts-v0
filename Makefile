.PHONY: build test clean fmt clippy check-size

# Default target
all: build

# Build BPF program
build:
	cargo build-bpf --manifest-path program/Cargo.toml -- --release

# Run all tests
test:
	cargo test -- --nocapture

# Clean build artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt -- --check

# Run clippy
clippy:
	cargo clippy -- -D warnings

# Check BPF size
check-size: build
	@echo "Checking BPF size (must be <= 120 KiB)..."
	@du -b program/target/deploy/*.so | awk '$$1 > 122880 { print "ERROR: BPF program size exceeds 120 KiB limit!"; exit 1 } { print "OK: BPF program size is " $$1 " bytes" }'

# Lint and format
lint: fmt clippy

# Full CI check
ci: lint test check-size