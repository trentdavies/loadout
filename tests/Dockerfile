FROM rust:bookworm

# Extra tools (git already included via buildpack-deps)
RUN apt-get update && apt-get install -y \
    openssh-server jq tree vim less bash-completion fzf zsh \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests and source
COPY Cargo.toml Cargo.loc[k] ./
COPY build.rs ./
COPY src/ ./src/

# Build with persistent cargo caches (registry + target dir survive across builds)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release \
    && cp target/release/equip /usr/local/bin/equip

# Symlink so equip is available at ~/.local/bin too
RUN mkdir -p /root/.local/bin && ln -s /usr/local/bin/equip /root/.local/bin/equip

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
    && chmod +x /tests/sandbox/runner.sh /tests/sandbox/setup.sh \
    && chmod +x /tests/sandbox/suite/*.sh 2>/dev/null || true

# Shell config: vi bindings, completions, equip completions
RUN printf '%s\n' \
    'set editing-mode vi' \
    'set keymap vi-command' \
    '$if mode=vi' \
    '  set show-mode-in-prompt on' \
    '  set vi-ins-mode-string \1\e[6 q\2' \
    '  set vi-cmd-mode-string \1\e[2 q\2' \
    '$endif' \
    > /root/.inputrc \
    && printf '%s\n' \
    'set -o vi' \
    '[ -f /usr/share/bash-completion/bash_completion ] && . /usr/share/bash-completion/bash_completion' \
    > /root/.bashrc \
    && equip completions bash --install

# Zsh config: vi bindings, fpath before compinit, equip completions
RUN mkdir -p /root/.zfunc \
    && equip completions zsh > /root/.zfunc/_equip \
    && printf '%s\n' \
    'bindkey -v' \
    'fpath=(/root/.zfunc $fpath)' \
    'autoload -Uz compinit && compinit' \
    > /root/.zshrc

# Convenience symlinks for sandbox exploration
RUN ln -s /tests/sandbox/runner.sh ~/run-tests \
    && ln -s /tests/sandbox/setup.sh ~/setup.sh

# SSH port for sandbox-ssh mode
EXPOSE 22

# Default: interactive shell. Callers override CMD:
#   just harness     → /tests/harness/runner.sh
#   just sandbox-test → /root/run-tests
#   just sandbox-ssh  → /usr/sbin/sshd -D
CMD ["bash"]
