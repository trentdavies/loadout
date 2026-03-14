FROM rust:latest

# Tools for interactive exploration
RUN apt-get update && apt-get install -y \
    jq tree vim less git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo manifests first for layer caching
COPY Cargo.toml Cargo.loc[k] ./

# Copy source code
COPY src/ ./src/

# Build skittle
RUN mkdir -p /root/.local/bin \
    && cargo build --release && cp target/release/skittle /root/.local/bin/skittle

# Clean build dir to save space
RUN rm -rf /build/target

# Set up realistic home directory layout
ENV HOME=/root
WORKDIR /root

# skittle on PATH via ~/.local/bin
ENV PATH="/root/.local/bin:${PATH}"

# XDG — mirrors real ~/.local/share/skittle
ENV XDG_DATA_HOME=/root/.local/share

# Simulate agent config directories
RUN mkdir -p ~/.claude ~/.codex ~/.cursor

# Mock install targets that mirror real locations
RUN mkdir -p ~/.claude/skills ~/.codex/skills

# Copy test harness lib (assertions)
COPY tests/harness/lib.sh /tests/harness/lib.sh

# Copy sandbox test suite
COPY tests/sandbox/ /tests/sandbox/

# Copy fixtures
COPY tests/fixtures/ /tests/fixtures/

# Place runner and setup in home directory for easy access
RUN ln -s /tests/sandbox/runner.sh ~/run-tests \
    && ln -s /tests/sandbox/setup.sh ~/setup.sh

# Make scripts executable
RUN chmod +x /tests/sandbox/run /tests/sandbox/runner.sh /tests/sandbox/setup.sh \
    && chmod +x /tests/sandbox/suite/*.sh

# Default: interactive shell, not the test runner
CMD ["bash"]
