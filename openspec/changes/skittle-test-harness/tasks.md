## 1. Test Fixtures

- [x] 1.1 Create `tests/fixtures/single-skill/SKILL.md` with valid frontmatter (name: single-skill, description)
- [x] 1.2 Create `tests/fixtures/flat-skills/` with `explore/SKILL.md` and `apply/SKILL.md` (valid frontmatter, no toml)
- [x] 1.3 Create `tests/fixtures/plugin-source/plugin.toml` (name: test-plugin, version: 0.1.0) with `skills/explore/`, `skills/apply/` (including `scripts/run.sh`), and `skills/verify/` — each with valid SKILL.md
- [x] 1.4 Create `tests/fixtures/full-source/source.toml` (name: test-source, version: 1.0.0) with `test-plugin-a/` (plugin.toml + skill-one, skill-two) and `test-plugin-b/` (plugin.toml + skill-three)
- [x] 1.5 Create `tests/fixtures/invalid/` with `no-frontmatter/SKILL.md` (no YAML), `bad-name/SKILL.md` (name doesn't match dir), and `empty-dir/`

## 2. Assertion Library

- [x] 2.1 Create `tests/harness/lib.sh` with global pass/fail/skip counters and test name tracking
- [x] 2.2 Implement `assert_exit_code <expected> <command...>` — run command, check exit code, print PASS/FAIL
- [x] 2.3 Implement `assert_stdout_contains <pattern> <command...>` and `assert_stderr_contains <pattern> <command...>`
- [x] 2.4 Implement `assert_stdout_eq <expected> <command...>` for exact stdout matching
- [x] 2.5 Implement `assert_file_exists`, `assert_file_not_exists`, `assert_dir_exists`, `assert_file_contains`
- [x] 2.6 Implement `assert_json_field <json_string> <jq_path> <expected>` using jq
- [x] 2.7 Implement `skip_if_no_network` helper that checks `SKIP_NETWORK=1` and increments skip counter

## 3. Test Runner

- [x] 3.1 Create `tests/harness/runner.sh` — discover `suite/*.sh` files, source each, execute all `test_` functions
- [x] 3.2 Add `--suite <number>` flag to run a specific suite file only
- [x] 3.3 Add summary output at end: "Results: X passed, Y failed, Z skipped, W total"
- [x] 3.4 Exit with code 0 if all pass, 1 if any fail

## 4. Environment Setup

- [x] 4.1 Create `tests/harness/setup.sh` with `reset_environment` function: wipe XDG dirs, recreate empty mock targets at `/tmp/test-targets/claude/` and `/tmp/test-targets/codex/`
- [x] 4.2 Add `setup_source_and_targets` helper: runs `skittle init`, adds plugin-source fixture as source, adds claude and codex mock targets
- [x] 4.3 Add `FIXTURES_DIR` and `SKITTLE` environment variable exports (paths to fixtures and binary)

## 5. Test Suites

- [x] 5.1 Create `tests/harness/suite/00_cli_framework.sh` — tests: --help, -h, help at top level and subcommands, unknown command error, install with no flags errors, global flags (--json, -n, -q, -v)
- [x] 5.2 Create `tests/harness/suite/01_config.sh` — tests: init creates config, init idempotent, config show, config show --json, cache show, cache clean empty
- [x] 5.3 Create `tests/harness/suite/02_source_management.sh` — tests: add local source, add git source (@network), remove source, list (empty + populated), show, update, duplicate name error
- [x] 5.4 Create `tests/harness/suite/03_source_detection.sh` — tests: single file, flat dir, plugin dir, full source, unrecognizable dir error, invalid skill warnings
- [x] 5.5 Create `tests/harness/suite/04_plugin_system.sh` — tests: plugin list, plugin list --source, plugin show, implicit plugin naming
- [x] 5.6 Create `tests/harness/suite/05_local_registry.sh` — tests: registry.json exists after add, cache dir mirrors source, short-form skill identity, ambiguous identity error
- [x] 5.7 Create `tests/harness/suite/06_target_management.sh` — tests: add claude/codex targets, remove target (preserves dir), list targets, show target, unknown agent type error
- [x] 5.8 Create `tests/harness/suite/07_target_adapters.sh` — tests: claude adapter SKILL.md + scripts copy, codex adapter, custom TOML adapter paths
- [ ] 5.9 Create `tests/harness/suite/08_skill_operations.sh` — tests: skill list, skill list --plugin, skill list --source, skill show, invalid skill skipped with warning
- [ ] 5.10 Create `tests/harness/suite/09_install_engine.sh` — tests: install --all, --skill, --plugin, --bundle, --target, uninstall --skill/--bundle, dry run (-n), idempotent install
- [ ] 5.11 Create `tests/harness/suite/10_bundle_management.sh` — tests: create, delete, list, show, add skills, drop skills, install bundle, swap bundles, active bundle tracking
- [ ] 5.12 Create `tests/harness/suite/11_end_to_end.sh` — full lifecycle: init → source add → target add → bundle create → bundle add → install --bundle → status → swap → uninstall → source remove → cache clean

## 6. Dockerfile

- [ ] 6.1 Create `tests/Dockerfile` with `rust:latest` base, install jq, copy Cargo.toml + Cargo.lock first for layer caching
- [ ] 6.2 Add cargo build step with graceful failure handling (wrapper script checks if binary exists before running tests)
- [ ] 6.3 Set XDG env vars: `XDG_CONFIG_HOME=/tmp/test-config`, `XDG_DATA_HOME=/tmp/test-data`
- [ ] 6.4 Copy fixtures and harness into container, create mock target dirs
- [ ] 6.5 Set CMD to run `tests/harness/runner.sh`

## 7. Verify Harness

- [ ] 7.1 Run `docker build -t skittle-test -f tests/Dockerfile .` — verify build fails gracefully (no Rust source yet) and reports "BUILD FAILED"
- [ ] 7.2 Create minimal `Cargo.toml` + `src/main.rs` (just exits 1) so the binary compiles, rebuild — verify all tests run and all fail
- [ ] 7.3 Verify `--suite` flag runs only the specified suite
- [ ] 7.4 Verify `SKIP_NETWORK=1` skips network tests
- [ ] 7.5 Verify summary counts are accurate (total = passed + failed + skipped)
