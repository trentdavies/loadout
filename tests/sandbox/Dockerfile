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
RUN cargo build --release && cp target/release/skittle /usr/local/bin/skittle

# Clean build dir to save space
RUN rm -rf /build/target

# XDG path — no SKIP_NETWORK, these tests hit the network
ENV XDG_DATA_HOME=/tmp/sandbox-data

# Create mock target directories
RUN mkdir -p /tmp/sandbox-targets/claude /tmp/sandbox-targets/codex

# Create local clone workspace
RUN mkdir -p /tmp/sandbox-local

# Copy test harness lib (assertions)
COPY tests/harness/lib.sh /tests/harness/lib.sh

# Copy sandbox test suite
COPY tests/sandbox/ /tests/sandbox/

# Copy fixtures (used by harness lib reference, may be useful for exploration)
COPY tests/fixtures/ /tests/fixtures/

# Make scripts executable
RUN chmod +x /tests/sandbox/run /tests/sandbox/runner.sh /tests/sandbox/setup.sh \
    && chmod +x /tests/sandbox/suite/*.sh

CMD ["/tests/sandbox/runner.sh"]
