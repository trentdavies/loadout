## 1. CLI Code (`src/cli/mod.rs`)

- [ ] 1.1 Update `Cli` struct `about` string to remove "source, cache" phrasing
- [ ] 1.2 Update `Command::Add` doc comment from "shorthand" to "Add a skill source"
- [ ] 1.3 Change `Command::List` to accept optional positional `name: Option<String>` arg
- [ ] 1.4 Add `Command::Remove { name: String, force: bool }` variant
- [ ] 1.5 Add `Command::Update { name: Option<String> }` variant
- [ ] 1.6 Delete `Command::Source`, `Command::Plugin`, `Command::Skill`, `Command::Cache` variants
- [ ] 1.7 Delete `SourceCommand`, `PluginCommand`, `SkillCommand`, `CacheCommand` enums
- [ ] 1.8 Update `Command::List` handler: branch on `name` — `None` lists all skills, `Some(identity)` shows skill details
- [ ] 1.9 Add `Command::Remove` handler (copy logic from `SourceCommand::Remove`)
- [ ] 1.10 Add `Command::Update` handler (copy logic from `SourceCommand::Update`)
- [ ] 1.11 Delete `Command::Source`, `Command::Plugin`, `Command::Skill`, `Command::Cache` match arms
- [ ] 1.12 Delete `dir_size`/`format_size` helper functions and their unit tests
- [ ] 1.13 Update user-facing strings: replace all references to `source add`, `skill list`, etc. with new command names

## 2. Rust Tests

- [ ] 2.1 `tests/cli_flags.rs`: Delete `parse_source_add_with_name` test
- [ ] 2.2 `tests/cli_flags.rs`: Rename `parse_add_shorthand` → `parse_add`, `parse_list_shorthand` → `parse_list` with updated match
- [ ] 2.3 `tests/cli_flags.rs`: Add `parse_list_with_name`, `parse_remove`, `parse_update` tests
- [ ] 2.4 `tests/integration_archive_and_shortcuts.rs`: Rename shorthand tests, update `List` match patterns
- [ ] 2.5 `tests/smoke_test.rs`: Update workflow comment
- [ ] 2.6 Run `cargo build` and `cargo test` — verify all pass

## 3. Harness Tests

- [ ] 3.1 `tests/harness/setup.sh`: Change `source add` → `add` in `setup_source_and_targets()`
- [ ] 3.2 `tests/harness/suite/00_cli_framework.sh`: Remove source/skill/plugin/cache help tests, update help assertions for new commands
- [ ] 3.3 `tests/harness/suite/02_source_management.sh`: Replace all `source add/remove/list/show/update` with `add/remove/update/list`
- [ ] 3.4 `tests/harness/suite/03_source_detection.sh`: Replace `source add` → `add`, `skill list`/`plugin list` → `list`
- [ ] 3.5 Delete `tests/harness/suite/04_plugin_system.sh`
- [ ] 3.6 `tests/harness/suite/05_local_registry.sh`: Replace `source add` → `add`, `source remove` → `remove`, `skill show` → `list`
- [ ] 3.7 `tests/harness/suite/08_skill_operations.sh`: Replace `skill list/show` → `list`, remove filter tests, `source add` → `add`
- [ ] 3.8 `tests/harness/suite/11_end_to_end.sh`: Replace all old commands, remove `cache clean` steps

## 4. OpenSpec Specs

- [ ] 4.1 Delete `openspec/specs/source-management/` directory
- [ ] 4.2 Delete `openspec/specs/plugin-system/` directory
- [ ] 4.3 Delete `openspec/specs/skill-operations/` directory
- [ ] 4.4 Update `openspec/specs/cli-framework/spec.md` with new command list and help scenarios
- [ ] 4.5 Update `openspec/specs/config-management/spec.md` — remove cache clean/show requirements
- [ ] 4.6 Update `openspec/specs/local-registry/spec.md` — replace command references
- [ ] 4.7 Update `openspec/specs/test-suites/spec.md` — remove suite 04, update command names throughout

## 5. Verification

- [ ] 5.1 `cargo build` compiles cleanly
- [ ] 5.2 `cargo test` — all Rust tests pass
- [ ] 5.3 Harness tests pass (build binary + run `./tests/harness/runner.sh`)
- [ ] 5.4 Manual smoke: `skittle --help` shows new command set, `skittle source` errors
