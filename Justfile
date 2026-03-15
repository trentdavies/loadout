# Skittle task runner — `just --list` to see all tasks

# Run all Rust tests (unit + integration)
test:
    LOADOUT_NON_INTERACTIVE=1 cargo test

# Build debug binary
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Run the shell test harness in Docker (offline, no network)
harness:
    docker buildx build --load -f tests/Dockerfile -t loadout-harness . && docker run --rm loadout-harness /tests/harness/runner.sh

# Launch sandbox container (detached, interactive)
sandbox:
    tests/sandbox/run

# Run sandbox test suite (network-dependent, real git repos)
sandbox-test:
    tests/sandbox/run --test

# Run sandbox tests and keep container alive for exploration
sandbox-keep:
    tests/sandbox/run --keep-alive

# Launch sandbox with SSH server and mounted git credentials
sandbox-ssh port="2222":
    tests/sandbox/run --ssh {{port}}

# Run sandbox tests with SSH git auth available
sandbox-test-ssh:
    tests/sandbox/run --test --ssh

# SSH into the running sandbox container
sandbox-connect port="2222":
    ssh -o StrictHostKeyChecking=no -p {{port}} root@localhost

# Connect to a running sandbox container (docker exec)
sandbox-exec:
    docker exec -it loadout-sandbox bash

# Stop and remove sandbox container
sandbox-clean:
    docker rm -f loadout-sandbox

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
