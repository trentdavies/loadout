# Skittle task runner — `just --list` to see all tasks

# Run all Rust tests (unit + integration)
test:
    SKITTLE_NON_INTERACTIVE=1 cargo test

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

# Auto-fix formatting and clippy warnings
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

# Tag a release (bumps version in Cargo.toml, commits, and tags)
tag version:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
    if [ "{{version}}" = "$current" ]; then
        echo "Version is already $current"
        exit 1
    fi
    sed -i '' "s/^version = \"$current\"/version = \"{{version}}\"/" Cargo.toml
    cargo check --quiet
    git add Cargo.toml Cargo.lock
    git commit -m "release v{{version}}"
    git tag -a "v{{version}}" -m "v{{version}}"
    echo "Tagged v{{version}}. Push with: git push && git push --tags"

# Publish to crates.io (run `just tag <version>` first)
publish:
    cargo publish
