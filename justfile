# Justfile for rhythm-open-exchange

# Use PowerShell on Windows
set shell := ["pwsh", "-c"]

# Default recipe
default: qa

# Run all quality assurance checks (mimics CI pipeline)
qa: check fmt clippy test
    @echo "✓ All QA checks passed!"

# Check compilation
check:
    cargo check --all-targets

# Format check
fmt:
    cargo fmt --check

# Clippy lints
clippy:
    cargo clippy --all-targets -- -D warnings

# Run tests
test:
    cargo test

# Format code (fix)
fix-fmt:
    cargo fmt

# Run benchmarks
bench:
    cargo bench

# Build release
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean
