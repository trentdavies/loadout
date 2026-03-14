# Skittle task runner — `just --list` to see all tasks

# Run all Rust tests (unit + integration)
test:
    cargo test

# Build debug binary
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Run the shell test harness in Docker (offline, no network)
harness:
    DOCKER_BUILDKIT=0 docker build -f tests/Dockerfile -t skittle-harness . && docker run --rm skittle-harness

# Launch sandbox container (detached, interactive)
sandbox:
    tests/sandbox/run

# Run sandbox test suite (network-dependent, real git repos)
sandbox-test:
    tests/sandbox/run --test

# Run sandbox tests and keep container alive for exploration
sandbox-keep:
    tests/sandbox/run --keep-alive

# Connect to a running sandbox container
sandbox-exec:
    docker exec -it skittle-sandbox bash

# Stop and remove sandbox container
sandbox-clean:
    docker rm -f skittle-sandbox

# Run all tests: Rust + harness
test-all: test harness

# Check formatting and clippy
check:
    cargo fmt -- --check
    cargo clippy -- -D warnings
