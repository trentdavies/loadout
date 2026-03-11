## Why

The `skittle-cli-v1` change defines 11 capabilities and 49 implementation tasks. Before writing any implementation code, we need a comprehensive external test harness that exercises the `skittle` binary as a black box — validating every command, subcommand, flag, and workflow end-to-end. This enables red-green development: all tests fail initially, then pass incrementally as the CLI is implemented. The harness runs in Docker to guarantee isolation from the host system (no XDG pollution, no accidental writes to real agent configs).

## What Changes

- Dockerfile that builds the skittle binary and sets up an isolated test environment with controlled XDG paths
- Local skill fixture repository (`tests/fixtures/`) containing test sources in every supported structure: single SKILL.md, flat dir, plugin with plugin.toml, full multi-plugin source with source.toml
- Shell-based test runner (`tests/harness/`) that invokes the `skittle` binary and asserts on exit codes, stdout, stderr, and filesystem state
- Test suites covering all 11 capabilities from `skittle-cli-v1`: cli-framework, source-management, plugin-system, source-detection, local-registry, target-management, target-adapters, skill-operations, bundle-management, install-engine, config-management
- Tests against a real remote source (Anthropic's skills repository via git) to validate git-based source fetching
- Test targets: mock `.claude/` and `.codex/` directories inside the container to validate skill installation

## Capabilities

### New Capabilities
- `test-fixtures`: Local skill fixtures covering all source structure types (single file, flat dir, plugin, multi-plugin source) plus a reference to the Anthropic skills repo for remote source testing
- `test-harness`: Dockerized test runner that builds skittle, sets up isolated XDG/target paths, runs all test suites, and reports pass/fail with clear output
- `test-suites`: Individual test scripts covering every skittle command and its scenarios, mapped 1:1 to the `skittle-cli-v1` specs

### Modified Capabilities

(none)

## Impact

- New files: `Dockerfile`, `tests/harness/`, `tests/fixtures/`
- Depends on Docker being available on the host
- Depends on network access for cloning the Anthropic skills repo during test runs (can be cached)
- No impact on skittle source code — the harness is purely external
