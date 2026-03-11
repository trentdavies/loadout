## Why

The skittle CLI has full implementation coverage but significant gaps in `cargo test` coverage. Several core modules — source discovery, source fetching, manifest parsing, and output formatting — have zero unit tests. Most CLI subcommand handlers lack functional tests. Error paths, edge cases, and spec-defined behaviors go unverified. The Docker-based test harness covers these at the shell level, but `cargo test` should be the primary quality gate for Rust-level correctness.

## What Changes

- Add unit tests for all untested modules: `source/discover.rs`, `source/fetch.rs`, `source/manifest.rs`, `source/normalize.rs`, `source/detect.rs`, `output/mod.rs`, `config/mod.rs`
- Add unit tests for helper functions in `cli/mod.rs` (`dir_size`, `format_size`)
- Expand existing unit tests in `source/url.rs` (relative paths, home expansion, invalid URLs)
- Expand existing unit tests in `config/types.rs` (malformed TOML, missing fields, adapter defaults)
- Expand existing unit tests in `registry/mod.rs` (`find_plugin`, corrupted JSON, parse errors)
- Add functional integration tests exercising CLI subcommand handlers through the library API: source CRUD, target CRUD, plugin/skill queries, bundle lifecycle, config/cache operations, install with specific flags (`--skill`, `--plugin`, `--bundle`, `--target`)
- Add error-path tests: invalid inputs, missing files, permission scenarios, malformed data
- Add dry-run verification tests confirming no filesystem writes occur

## Capabilities

### New Capabilities

- `unit-test-coverage`: Unit tests for all source modules (detect, discover, fetch, manifest, normalize, url), config module, registry module, output module, and CLI helpers
- `functional-test-coverage`: Integration tests exercising CLI command handlers through the library API — source, target, plugin, skill, bundle, config, cache, install, and status operations
- `error-path-coverage`: Tests for error conditions, malformed inputs, missing files, and edge cases specified in the skittle-cli-v1 specs

### Modified Capabilities

_(none — no spec-level behavior changes, only test additions)_

## Impact

- **Code**: New test modules in `src/` (inline `#[cfg(test)]`) and new files in `tests/`
- **Dependencies**: May add `assert_cmd` or similar dev-dependency for CLI binary testing; otherwise only `tempfile` (already present)
- **CI**: `cargo test` runtime will increase but remains single-digit seconds
- **Risk**: Zero — tests are additive with no production code changes
