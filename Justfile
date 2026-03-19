# Skittle task runner — `just --list` to see all tasks

# Run all Rust tests (unit + integration)
test:
    EQUIP_NON_INTERACTIVE=1 cargo test

# Build debug binary
build:
    cargo build

# Build release binary
build-release:
    cargo build --release

# Install locally from source
install:
    cargo install --path .

# ── Docker sandbox ──────────────────────────────────────────────

image   := "equip-sandbox"
container := "equip-sandbox"

# Build the sandbox Docker image
[private]
sandbox-build:
    docker buildx build --load -f tests/Dockerfile -t {{image}} .

# Run the shell test harness in Docker (offline, no network)
harness: sandbox-build
    docker run --rm {{image}} /tests/harness/runner.sh

# Launch sandbox container (detached, interactive).  pass 'zsh' for zsh.
sandbox sh="bash": sandbox-build
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{container}} 2>/dev/null || true
    docker run -d --name {{container}} {{image}} sleep infinity >/dev/null
    echo ""
    echo "Sandbox running. Connect with:"
    echo ""
    echo "  docker exec -it {{container}} {{sh}}"
    echo ""
    echo "Inside the container:"
    echo "  equip                                 # on PATH (~/.local/bin)"
    echo "  ~/run-tests                           # run the full test suite"
    echo "  source ~/setup.sh && sandbox_init     # set up agents without running tests"
    echo ""
    echo "Cleanup: just sandbox-clean"

# Run sandbox test suite (network-dependent, real git repos)
sandbox-test: sandbox-build
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{container}} 2>/dev/null || true
    docker run --rm --name {{container}} {{image}} /root/run-tests

# Run sandbox tests and keep container alive for exploration.  pass 'zsh' for zsh.
sandbox-keep sh="bash": sandbox-build
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{container}} 2>/dev/null || true
    docker run -d --name {{container}} {{image}} \
        bash -c '/root/run-tests; echo ""; echo "Tests finished. Container alive for exploration."; exec sleep infinity' >/dev/null
    echo "Streaming test output (ctrl-c to stop following)..."
    echo ""
    docker logs -f {{container}} || true
    echo ""
    echo "Container is still running. Explore with:"
    echo ""
    echo "  docker exec -it {{container}} {{sh}}"
    echo ""
    echo "Cleanup: just sandbox-clean"

# Launch sandbox with SSH server and mounted git credentials.  pass 'zsh' for zsh.
sandbox-ssh port="2222" sh="bash": sandbox-build
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{container}} 2>/dev/null || true
    MOUNT_ARGS=(-v "$HOME/.ssh:/tmp/host-ssh:ro")
    [ -f "$HOME/.gitconfig" ] && MOUNT_ARGS+=(-v "$HOME/.gitconfig:/root/.gitconfig:ro")
    SSH_INIT='
      if [ -d /tmp/host-ssh ]; then
        cp /tmp/host-ssh/config /root/.ssh/config 2>/dev/null || true
        cat /tmp/host-ssh/*.pub > /root/.ssh/authorized_keys 2>/dev/null || true
        for f in /tmp/host-ssh/*; do
          name=$(basename "$f")
          case "$name" in known_hosts|authorized_keys|*.pub|config) continue ;; esac
          [ -f "$f" ] && cp "$f" /root/.ssh/ && chmod 600 "/root/.ssh/$name"
        done
        chmod 600 /root/.ssh/authorized_keys 2>/dev/null || true
        chmod 600 /root/.ssh/config 2>/dev/null || true
      fi
    '
    docker run -d --name {{container}} \
        -p {{port}}:22 \
        "${MOUNT_ARGS[@]}" \
        {{image}} bash -c "$SSH_INIT exec /usr/sbin/sshd -D" >/dev/null
    echo ""
    echo "Sandbox running with SSH + git credentials. Connect with:"
    echo ""
    echo "  ssh -o StrictHostKeyChecking=no -p {{port}} root@localhost"
    echo "  docker exec -it {{container}} {{sh}}"
    echo ""
    echo "Cleanup: just sandbox-clean"

# Run sandbox tests with SSH git auth available
sandbox-test-ssh: sandbox-build
    #!/usr/bin/env bash
    set -euo pipefail
    docker rm -f {{container}} 2>/dev/null || true
    MOUNT_ARGS=(-v "$HOME/.ssh:/tmp/host-ssh:ro")
    [ -f "$HOME/.gitconfig" ] && MOUNT_ARGS+=(-v "$HOME/.gitconfig:/root/.gitconfig:ro")
    SSH_INIT='
      if [ -d /tmp/host-ssh ]; then
        cp /tmp/host-ssh/config /root/.ssh/config 2>/dev/null || true
        cat /tmp/host-ssh/*.pub > /root/.ssh/authorized_keys 2>/dev/null || true
        for f in /tmp/host-ssh/*; do
          name=$(basename "$f")
          case "$name" in known_hosts|authorized_keys|*.pub|config) continue ;; esac
          [ -f "$f" ] && cp "$f" /root/.ssh/ && chmod 600 "/root/.ssh/$name"
        done
        chmod 600 /root/.ssh/authorized_keys 2>/dev/null || true
        chmod 600 /root/.ssh/config 2>/dev/null || true
      fi
    '
    docker run --rm --name {{container}} \
        "${MOUNT_ARGS[@]}" \
        {{image}} bash -c "$SSH_INIT exec /root/run-tests"

# SSH into the running sandbox container
sandbox-connect port="2222":
    ssh -o StrictHostKeyChecking=no -p {{port}} root@localhost

# Connect to a running sandbox container (docker exec).  pass 'zsh' for zsh.
sandbox-exec sh="bash":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker ps --filter name=^/{{container}}$ --filter status=running | grep -q '{{container}}'; then
        echo "Sandbox is not running; starting it..."
        just sandbox >/dev/null
    fi
    docker exec -it {{container}} {{sh}}

# Stop and remove sandbox container
sandbox-clean:
    docker rm -f {{container}}

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

# Install repo-managed git hooks for this worktree
install-hooks:
    git config core.hooksPath .githooks
    chmod +x .githooks/pre-commit

# Release: bump version, tag, push — CI builds binaries and updates the Homebrew tap
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
    if [ "{{version}}" = "$current" ]; then
        echo "error: version is already $current"
        exit 1
    fi
    sed -i '' "s/^version = \"$current\"/version = \"{{version}}\"/" Cargo.toml
    cargo check --quiet
    EQUIP_NON_INTERACTIVE=1 cargo test --quiet
    git add Cargo.toml Cargo.lock
    git commit -m "release v{{version}}"
    git tag -a "v{{version}}" -m "v{{version}}"
    git push && git push --tags
    echo ""
    echo "v{{version}} pushed — CI will build binaries and update homebrew-tap."
    echo "  Watch: gh run watch"

# Publish to crates.io (optional, after CI release completes)
publish:
    cargo publish
