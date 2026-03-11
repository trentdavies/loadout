## ADDED Requirements

### Requirement: Dockerfile builds skittle and runs tests
The `tests/Dockerfile` SHALL build the skittle binary from source using `rust:latest` as the base image, then run the test harness. The build step SHALL be separated from the test step so that build failures are reported gracefully.

#### Scenario: Successful build and test run
- **WHEN** `docker build -t skittle-test -f tests/Dockerfile .` completes and `docker run --rm skittle-test` is executed
- **THEN** the container SHALL build skittle, run all test suites, and output pass/fail results

#### Scenario: Build failure
- **WHEN** the Rust source does not compile (expected during red-green development)
- **THEN** the harness SHALL report "BUILD FAILED — skittle binary not available" and exit with a non-zero code without running any tests

### Requirement: Controlled XDG environment
The container SHALL set `XDG_CONFIG_HOME=/tmp/test-config` and `XDG_DATA_HOME=/tmp/test-data` so that all skittle operations use predictable, inspectable paths inside the container.

#### Scenario: Skittle uses test XDG paths
- **WHEN** `skittle init` is run inside the container
- **THEN** the config SHALL be created at `/tmp/test-config/skittle/config.toml`

#### Scenario: Registry uses test data path
- **WHEN** sources are added
- **THEN** cached content SHALL appear under `/tmp/test-data/skittle/`

### Requirement: Mock target directories
The container SHALL create mock target directories at `/tmp/test-targets/claude/` and `/tmp/test-targets/codex/` to simulate agent installations without real agent software.

#### Scenario: Claude mock target exists
- **WHEN** the test harness starts
- **THEN** `/tmp/test-targets/claude/` SHALL exist and be empty

#### Scenario: Skills install into mock targets
- **WHEN** a skill is installed to the claude mock target
- **THEN** the skill SHALL appear at `/tmp/test-targets/claude/skills/<name>/SKILL.md`

### Requirement: Test runner discovers and executes suites
The `tests/harness/runner.sh` SHALL discover all `*.sh` files in `tests/harness/suite/`, source them in alphanumeric order, and execute all functions prefixed with `test_`. It SHALL print a summary at the end: total tests, passed, failed.

#### Scenario: Run all suites
- **WHEN** `runner.sh` is invoked with no arguments
- **THEN** all suite files SHALL be sourced and all `test_` functions executed

#### Scenario: Run specific suite
- **WHEN** `runner.sh --suite 02` is invoked
- **THEN** only `02_source_management.sh` SHALL be executed

#### Scenario: Summary output
- **WHEN** all tests complete
- **THEN** the runner SHALL print: "Results: X passed, Y failed, Z total" and exit with code 0 if all pass, 1 if any fail

### Requirement: Assertion library
The `tests/harness/lib.sh` SHALL provide assertion functions: `assert_exit_code`, `assert_stdout_contains`, `assert_stderr_contains`, `assert_stdout_eq`, `assert_file_exists`, `assert_file_not_exists`, `assert_file_contains`, `assert_dir_exists`, `assert_json_field`. Each assertion SHALL print PASS or FAIL with context (test name, expected, actual) and increment global counters.

#### Scenario: Passing assertion
- **WHEN** `assert_exit_code 0 true` is called
- **THEN** it SHALL print "PASS: ..." and increment the pass counter

#### Scenario: Failing assertion
- **WHEN** `assert_exit_code 0 false` is called
- **THEN** it SHALL print "FAIL: ... expected exit code 0, got 1" and increment the fail counter without aborting

### Requirement: Per-suite isolation via setup
The `tests/harness/setup.sh` SHALL provide a `reset_environment` function that: wipes `$XDG_CONFIG_HOME/skittle/`, wipes `$XDG_DATA_HOME/skittle/`, and recreates empty mock target directories. Each test function SHALL call `reset_environment` before executing.

#### Scenario: Tests are isolated
- **WHEN** test A creates a config and test B runs after it
- **THEN** test B SHALL start with a clean environment (no config, no registry, empty targets)

### Requirement: Network test gating
Tests that require network access (git clone of remote repos) SHALL be skipped when the `SKIP_NETWORK` environment variable is set to `1`.

#### Scenario: Skip network tests
- **WHEN** `docker run --rm -e SKIP_NETWORK=1 skittle-test` is executed
- **THEN** tests tagged with `@network` SHALL be skipped and reported as "SKIP" in the summary

#### Scenario: Network tests run by default
- **WHEN** `docker run --rm skittle-test` is executed without `SKIP_NETWORK`
- **THEN** network-dependent tests SHALL execute normally

### Requirement: Docker layer caching for fast rebuilds
The Dockerfile SHALL copy `Cargo.toml` and `Cargo.lock` first, run `cargo fetch`, then copy the source. This ensures dependency downloads are cached across rebuilds when only source code changes.

#### Scenario: Rebuild after source change
- **WHEN** only `.rs` files change and `docker build` is run again
- **THEN** the dependency download layer SHALL be cached and only compilation runs
