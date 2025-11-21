# Ultrasearch Developer Workflows

set shell := ["bash", "-c"]

# Default: run all checks
default: all

# Run all quality gates (format, lint, test, build)
all: fmt lint test build

# Check code formatting
fmt:
    cargo fmt --all -- --check

# Run clippy lints (strict)
lint:
    cargo clippy --all-targets -- -D warnings

# Run all tests
# Uses cargo-nextest if available, otherwise standard cargo test
test:
    if command -v cargo-nextest >/dev/null; then \
        cargo nextest run --all-targets; \
    else \
        cargo test --all-targets; \
    fi

# Check for compilation errors (fast)
check:
    cargo check --all-targets

# Build release binaries
build:
    cargo build --release

# Run 'ubs' (Ultimate Bug Scanner) on changed files if available
ubs:
    if command -v ubs >/dev/null; then \
        ubs $(git diff --name-only -- '*.rs'); \
    else \
        echo "ubs not installed, skipping"; \
    fi

# Clean artifacts
clean:
    cargo clean
