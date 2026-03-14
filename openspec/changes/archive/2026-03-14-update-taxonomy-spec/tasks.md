## 1. Restore --dry-run global flag

- [x] 1.1 Add `dry_run` field back to `Cli` struct as global flag (`-n` / `--dry-run`)
- [x] 1.2 Re-add `--dry-run` checks to additive commands: install, source add, source update, target add
- [x] 1.3 Make destructive commands ignore `--dry-run` (they already default to preview without `--force`)
- [x] 1.4 Handle `--dry-run` + `--force` on destructive commands: `--dry-run` wins
- [x] 1.5 Update cli_flags tests for `--dry-run` parsing
- [x] 1.6 Update functional_dry_run tests for new semantics

## 2. Archive support — SourceUrl

- [x] 2.1 Add `zip` crate dependency to Cargo.toml
- [x] 2.2 Add `SourceUrl::Archive(PathBuf)` variant to `src/source/url.rs`
- [x] 2.3 Update `SourceUrl::parse` to detect `.zip` and `.skill` extensions before Local/Git fallthrough
- [x] 2.4 Add `source_type()` and `url_string()` implementations for Archive variant
- [x] 2.5 Add `default_name()` for Archive (filename without extension)
- [x] 2.6 Add unit tests for Archive URL parsing (`.zip`, `.skill`, non-archive fallthrough)

## 3. Archive support — fetch

- [x] 3.1 Add `fetch_archive()` function to `src/source/fetch.rs` that extracts zip to cache dir
- [x] 3.2 Enforce size limit (100MB unpacked) and file count limit (10,000) during extraction
- [x] 3.3 Handle `.skill` files as zip format
- [x] 3.4 Wire Archive variant into `fetch()` dispatch
- [x] 3.5 Add unit tests: unpack valid zip, unpack .skill, file-not-found error, size limit, file count limit

## 4. .claude-plugin detection and metadata

- [x] 4.1 Add `.claude-plugin` check to detection priority in `src/source/detect.rs` (after plugin.toml, before FlatSkills)
- [x] 4.2 Add `load_claude_plugin_metadata()` to `src/source/manifest.rs` — defensive parsing, warnings on malformed files
- [x] 4.3 Update `src/source/normalize.rs` to merge `.claude-plugin` metadata with `plugin.toml` (plugin.toml wins, .claude-plugin supplements)
- [x] 4.4 Add unit tests: detect with .claude-plugin only, detect with both plugin.toml and .claude-plugin, malformed .claude-plugin

## 5. Skill spec alignment

- [x] 5.1 Add kebab-case validation for skill names in `src/source/detect.rs` — warn and skip non-kebab-case skills
- [x] 5.2 Support `metadata.author` and `metadata.version` in SKILL.md frontmatter parsing
- [x] 5.3 Store author and version in `RegisteredSkill` if present
- [x] 5.4 Add unit tests: valid kebab-case, invalid name skipped with warning, optional metadata fields

## 6. CLI shortcut commands

- [x] 6.1 Add `Command::Add` variant that delegates to `source add` code path
- [x] 6.2 Add `Command::List` variant that delegates to `skill list` code path
- [x] 6.3 Update `Command::Init` to accept optional `url` argument
- [x] 6.4 Implement `init [url]`: clone/copy URL contents into `~/.local/share/skittle/` cache, then register as source
- [x] 6.5 Add cli_flags tests for `skittle add`, `skittle list`, `skittle init <url>` parsing

## 7. Spec sync

- [x] 7.1 Sync delta specs to main specs using `openspec sync`
- [x] 7.2 Verify all main specs reflect the changes from this change

## 8. Integration tests

- [x] 8.1 Add test fixture: valid `.zip` archive containing a plugin with skills
- [x] 8.2 Add test fixture: `.skill` file containing a single AgentSkill
- [x] 8.3 Add test fixture: directory with `.claude-plugin` file
- [x] 8.4 Integration test: `source add` with `.zip` file end-to-end
- [x] 8.5 Integration test: `source add` with `.skill` file end-to-end
- [x] 8.6 Integration test: `source add` with `.claude-plugin` directory
- [x] 8.7 Integration test: `skittle add` shorthand delegates correctly
- [x] 8.8 Integration test: `skittle list` shorthand delegates correctly
- [x] 8.9 Integration test: `skittle init <url>` populates cache and registers source
