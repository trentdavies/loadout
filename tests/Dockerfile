# ── Build stage ──────────────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /build

# Cache dependency build: copy manifests first, build a dummy, then copy real src
COPY Cargo.toml Cargo.loc[k] ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release 2>/dev/null; rm -rf src

COPY src/ ./src/
RUN cargo build --release

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssh-server jq tree vim less git \
    && rm -rf /var/lib/apt/lists/*

# SSH setup
RUN mkdir -p /run/sshd \
    && echo 'root:loadout' | chpasswd \
    && sed -i 's/#PermitRootLogin.*/PermitRootLogin yes/' /etc/ssh/sshd_config \
    && sed -i 's/#PasswordAuthentication.*/PasswordAuthentication yes/' /etc/ssh/sshd_config \
    && sed -i 's/#AllowAgentForwarding.*/AllowAgentForwarding yes/' /etc/ssh/sshd_config

# Copy binary from builder — available at both locations
COPY --from=builder /build/target/release/loadout /usr/local/bin/loadout
RUN mkdir -p /root/.local/bin && ln -s /usr/local/bin/loadout /root/.local/bin/loadout

# Environment
ENV HOME=/root
ENV XDG_DATA_HOME=/root/.local/share
ENV PATH="/root/.local/bin:${PATH}"
WORKDIR /root

# Simulate agent config directories
RUN mkdir -p ~/.claude ~/.codex ~/.cursor

# Mock install targets (harness uses these)
RUN mkdir -p /tmp/test-targets/claude /tmp/test-targets/codex

# Copy full test directory and fixtures
COPY tests/ /tests/

# Make scripts executable
RUN chmod +x /tests/harness/runner.sh \
    && chmod +x /tests/sandbox/run /tests/sandbox/runner.sh /tests/sandbox/setup.sh \
    && chmod +x /tests/sandbox/suite/*.sh 2>/dev/null || true

# Convenience symlinks for sandbox exploration
RUN ln -s /tests/sandbox/runner.sh ~/run-tests \
    && ln -s /tests/sandbox/setup.sh ~/setup.sh

# SSH port for sandbox-ssh mode
EXPOSE 22

# Default: interactive shell. Callers override CMD:
#   just harness        → /tests/harness/runner.sh
#   sandbox/run --test  → /root/run-tests
#   sandbox-ssh         → /usr/sbin/sshd -D
CMD ["bash"]
