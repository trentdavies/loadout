FROM rust:bookworm

# Extra tools (git already included via buildpack-deps)
RUN apt-get update && apt-get install -y \
    openssh-server jq tree vim less \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests and source
COPY Cargo.toml Cargo.loc[k] ./
COPY src/ ./src/

# Build with persistent cargo caches (registry + target dir survive across builds)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release \
    && cp target/release/loadout /usr/local/bin/loadout

# Symlink so loadout is available at ~/.local/bin too
RUN mkdir -p /root/.local/bin && ln -s /usr/local/bin/loadout /root/.local/bin/loadout

# SSH setup — key-based auth only (keys mounted at runtime via --ssh)
RUN mkdir -p /run/sshd /root/.ssh \
    && sed -i 's/#PermitRootLogin.*/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config \
    && sed -i 's/#PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config \
    && sed -i 's/#AllowAgentForwarding.*/AllowAgentForwarding yes/' /etc/ssh/sshd_config \
    && ssh-keyscan github.com >> /root/.ssh/known_hosts 2>/dev/null \
    && chmod 700 /root/.ssh && chmod 644 /root/.ssh/known_hosts

# Environment
ENV HOME=/root
ENV XDG_DATA_HOME=/root/.local/share
ENV PATH="/root/.local/bin:${PATH}"
WORKDIR /root

# Simulate agent config directories
RUN mkdir -p ~/.claude ~/.codex ~/.cursor

# Mock install agents (harness uses these)
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
