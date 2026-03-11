## Context

Skittle's `cargo test` suite has 5 integration test files and 7 unit tests in `registry/mod.rs`, plus 11 URL-parsing unit tests in `source/url.rs` and 6 config deserialization tests in `config/types.rs`. Several core modules have zero unit test coverage: `source/discover.rs`, `source/fetch.rs`, `source/manifest.rs`, `output/mod.rs`. CLI command handlers are exercised only through the Docker shell harness, not through `cargo test`. Error paths and edge cases specified in the skittle-cli-v1 specs are largely untested at the Rust level.

## Goals / Non-Goals

**Goals:**
- Every public function in every module has at least one unit test for its happy path
- Error paths (invalid input, missing files, malformed data) have explicit tests
- CLI command handlers are exercised through library-level functional tests
- Edge cases called out in skittle-cli-v1 specs have corresponding `cargo test` coverage
- Tests run fast (no network, no Docker, no git clone) using fixtures and tempdir isolation

**Non-Goals:**
- Achieving a specific coverage percentage target
- Testing the Docker harness or shell test suites
- Testing the `main.rs` binary entry point (clap parsing is already tested)
- Mocking external services (git2, reqwest) — test local/filesystem paths only
- Testing permission-denied or disk-full scenarios (platform-dependent, unreliable in CI)

## Decisions

### 1. Unit tests go inline, functional tests go in `tests/`

**Decision**: Unit tests use `#[cfg(test)] mod tests` inside each source file. Functional tests that exercise multi-module workflows go in `tests/` as integration test files.

**Rationale**: This is standard Rust convention. Inline unit tests can access private functions. Integration tests in `tests/` verify the public API surface as a consumer would.

**Alternative considered**: All tests in `tests/` — rejected because it prevents testing private helpers and creates unnecessary coupling to the public API for pure-unit scenarios.

### 2. Test organization mirrors module structure

**Decision**: Each source module gets its own `#[cfg(test)]` block. New integration test files are organized by capability area:
- `tests/functional_source_ops.rs` — source add/remove/list/show/update through library API
- `tests/functional_target_ops.rs` — target add/remove/list/show/detect through library API
- `tests/functional_skill_plugin_ops.rs` — skill/plugin list/show through library API
- `tests/functional_install_ops.rs` — install with `--skill`, `--plugin`, `--bundle`, `--target` flags
- `tests/functional_status_config_cache.rs` — status, config show, cache show/clean

**Rationale**: Matches the existing test file naming pattern (`integration_*.rs`). Keeps test files focused and scannable.

**Alternative considered**: One large `functional_all.rs` — rejected because it becomes unwieldy and makes targeted test runs harder.

### 3. Use existing fixtures, add targeted new ones

**Decision**: Reuse `tests/fixtures/` (single-skill, flat-skills, plugin-source, full-source, invalid). Add new fixture variants only where gaps demand them (e.g., a manifest with missing required fields, a SKILL.md with edge-case frontmatter).

**Rationale**: Existing fixtures already cover the four source structure types. Avoid fixture sprawl.

### 4. No mocking framework — use filesystem isolation

**Decision**: Tests that need filesystem state create tempdirs via `tempfile::TempDir`. Tests that exercise fetch/git functionality only test the local-path codepath. Git clone/fetch paths are tested only if they can use a local bare repo fixture.

**Rationale**: Network-dependent tests are flaky. The local fetch path exercises the same normalize/detect logic. Git operations can be tested with a local bare repo created in a tempdir.

### 5. Output module tests capture stdout

**Decision**: Output tests construct an `Output` instance and verify behavior by checking what gets written. Since `Output` writes to stdout/stderr directly, tests either: (a) test the formatting logic in isolation, or (b) accept that some methods are thin wrappers over `println!`/`colored` and focus on flag-gating logic (quiet suppresses, verbose enables).

**Rationale**: Capturing stdout in Rust tests is possible but fragile. Testing the decision logic (does quiet suppress? does json mode emit valid JSON?) is more valuable than testing exact terminal output.

## Risks / Trade-offs

- **[Tests may be brittle to internal refactoring]** → Unit tests for private functions couple to implementation. Mitigation: Focus unit tests on behavior, not structure. Test "given X input, expect Y output" not "calls Z internally."
- **[Git codepaths have limited test coverage]** → We can't easily test git clone without network. Mitigation: Test local-path fetch thoroughly; accept that git clone is covered by the Docker harness.
- **[Test suite runtime increase]** → More tests = longer `cargo test`. Mitigation: All tests use tempdir (fast I/O), no network, no sleep. Expected total < 5 seconds.
