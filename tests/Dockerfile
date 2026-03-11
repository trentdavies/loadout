FROM rust:latest

# Install jq for JSON assertion tests
RUN apt-get update && apt-get install -y jq && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo manifests first for layer caching
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Copy source code
COPY src/ ./src/

# Build with graceful failure handling
RUN cargo build --release 2>/dev/null; \
    if [ -f target/release/skittle ]; then \
      cp target/release/skittle /usr/local/bin/skittle; \
      echo "BUILD OK — skittle binary installed"; \
    else \
      echo "BUILD FAILED — skittle binary not produced (tests will report this)"; \
    fi

# Set XDG env vars for test isolation
ENV XDG_CONFIG_HOME=/tmp/test-config
ENV XDG_DATA_HOME=/tmp/test-data
ENV SKIP_NETWORK=1

# Create mock target directories
RUN mkdir -p /tmp/test-targets/claude /tmp/test-targets/codex

# Copy test fixtures and harness
COPY tests/ /tests/

# Make test scripts executable
RUN chmod +x /tests/harness/runner.sh

CMD ["/tests/harness/runner.sh"]
