## 1. Project Bootstrap

- [x] 1.1 Initialize Cargo project (`cargo init --name skittle`), set up `Cargo.toml` with dependencies: clap (derive feature), serde (derive feature), toml, serde_json, reqwest (blocking feature), git2, dirs, colored, thiserror, anyhow, glob
- [x] 1.2 Create module structure: `src/{cli, registry, source, target, bundle, config, output}` with `mod.rs` files and `lib.rs` root
- [x] 1.3 Set up `main.rs` with clap `#[derive(Parser)]` top-level command enum and global flags (-n, -v, -q, --json, --color, --config)
- [x] 1.4 Wire up help at every level (clap's `help` subcommand + `-h` + `--help`) and exit codes (0 success, non-zero error)

## 2. Config System

- [x] 2.1 Define config types in `src/config/types.rs`: `Config`, `SourceConfig`, `TargetConfig`, `AdapterConfig`, `BundleConfig` with serde derives
- [x] 2.2 Implement config loading from XDG path (`~/.config/skittle/config.toml`) with `--config` override, auto-create directories on first use
- [x] 2.3 Implement `skittle init` — create default config with commented examples
- [ ] 2.4 Implement `skittle config show` (text + --json) and `skittle config edit` ($EDITOR/$VISUAL/vi fallback)

## 3. Output Formatting

- [ ] 3.1 Create output module supporting text (colored) and JSON modes, respecting --color, --quiet, --verbose, and NO_COLOR env var
- [ ] 3.2 Define output helpers: table formatter, tree formatter, status/success/error/warning message helpers

## 4. Source Management

- [ ] 4.1 Implement URL resolution: parse local paths (file://, ~/, ./), git URLs (https://, git://), GitHub shorthand into a unified `SourceUrl` enum
- [ ] 4.2 Implement source fetching: `std::fs::copy` for local, `git2::Repository::clone` for git, `reqwest` for HTTP URLs
- [ ] 4.3 Implement progressive source detection: single file → source.toml → plugin.toml → flat skill dirs → single SKILL.md → error
- [ ] 4.4 Implement source normalization: wrap detected structure into canonical Source > Plugin > Skill hierarchy with implicit naming
- [ ] 4.5 Implement `skittle source add <url> [--name]` — resolve, fetch, detect, normalize, cache, register in config
- [ ] 4.6 Implement `skittle source remove <name>` with --force for sources with installed skills
- [ ] 4.7 Implement `skittle source list` (table: name, URL, plugin count, last updated)
- [ ] 4.8 Implement `skittle source show <name>` (details + plugin/skill tree)
- [ ] 4.9 Implement `skittle source update [name]` — re-fetch and update cache

## 5. Plugin System

- [ ] 5.1 Define plugin.toml and source.toml serde types with validation (required name, optional version/description/assets)
- [ ] 5.2 Implement plugin discovery: scan for plugin.toml in source subdirs, infer implicit plugins when no manifest
- [ ] 5.3 Implement asset discovery within plugins: scan for SKILL.md files, validate frontmatter (name, description required, name matches dir)
- [ ] 5.4 Implement `skittle plugin list [--source]` and `skittle plugin show <name>`

## 6. Local Registry

- [ ] 6.1 Implement registry index (`registry.json`): source/plugin/skill entries mapped to cached filesystem paths
- [ ] 6.2 Implement cache storage at `~/.local/share/skittle/sources/<name>/` with XDG override support
- [ ] 6.3 Implement skill identity resolution: `plugin/skill` short form, `source:plugin/skill` full form, ambiguity detection with helpful error messages
- [ ] 6.4 Implement `skittle skill list [--source] [--plugin]` and `skittle skill show <plugin/skill>` reading SKILL.md frontmatter for display

## 7. Target Management

- [ ] 7.1 Implement `skittle target add <agent> [path] [--scope] [--sync] [--name]` with defaults (machine scope → auto sync, repo scope → explicit sync)
- [ ] 7.2 Implement `skittle target remove <name>` (removes config, does not delete installed skills)
- [ ] 7.3 Implement `skittle target list` (table: name, agent, path, scope, sync, installed count)
- [ ] 7.4 Implement `skittle target show <name>` (config + installed skills list via adapter)
- [ ] 7.5 Implement `skittle target detect` — scan standard paths (~/.claude, ~/.codex, ~/.cursor, ./.claude, ./.codex) and prompt to add

## 8. Target Adapters

- [ ] 8.1 Define adapter trait: `install_skill`, `uninstall_skill`, `installed_skills` methods
- [ ] 8.2 Implement `claude` built-in adapter: copy skill dir to `{target}/skills/{name}/` (SKILL.md + scripts/ + references/ + assets/)
- [ ] 8.3 Implement `codex` built-in adapter: same layout as claude
- [ ] 8.4 Implement TOML-driven custom adapter: read `skill_dir`, `skill_file`, `format`, `copy_dirs` from config, validate format is "agentskills"
- [ ] 8.5 Implement adapter resolution: match target's agent type to built-in or custom adapter, error on unknown with available list

## 9. Install Engine

- [ ] 9.1 Implement `skittle install` requiring explicit flags (no flags → show help + exit non-zero)
- [ ] 9.2 Implement `skittle install --all [--target]` — install all configured skills to auto-sync targets (or specific target)
- [ ] 9.3 Implement `skittle install --skill <plugin/skill> [--target]` — install specific skill
- [ ] 9.4 Implement `skittle install --plugin <name> [--target]` — install all skills from plugin
- [ ] 9.5 Implement `skittle install --bundle <name> [--target]` — install bundle, track as active bundle on target
- [ ] 9.6 Implement idempotent install: skip if same version, update if newer
- [ ] 9.7 Implement dry run (`-n`): display planned operations without writing files
- [ ] 9.8 Implement `skittle uninstall` with --skill, --plugin, --bundle, --target flags
- [ ] 9.9 Implement active bundle tracking in registry (updated on install --bundle and bundle swap)

## 10. Bundle Management

- [ ] 10.1 Implement `skittle bundle create <name>` — validate name, add empty bundle to config
- [ ] 10.2 Implement `skittle bundle delete <name>` with --force for active bundles
- [ ] 10.3 Implement `skittle bundle list` (table: name, skill count, active targets)
- [ ] 10.4 Implement `skittle bundle show <name>` (skill list with plugin/source/version)
- [ ] 10.5 Implement `skittle bundle add <bundle> <skill...>` with glob support (e.g., `openspec/*`), validate skills exist in registry
- [ ] 10.6 Implement `skittle bundle drop <bundle> <skill...>`
- [ ] 10.7 Implement `skittle bundle swap <from> <to> [--target]` — clean replace: uninstall from, install to, update active bundle

## 11. Status & Cache

- [ ] 11.1 Implement `skittle status [--json]` — summary: source count, target count, installed skills, active bundles, outdated skills
- [ ] 11.2 Implement `skittle cache show` — cache path, total size, breakdown by source
- [ ] 11.3 Implement `skittle cache clean` — delete cached sources, clear registry, report space freed

## 12. Testing & Polish

- [ ] 12.1 Add integration tests: init, source add/remove/list/show/update, target add/remove/list/detect
- [ ] 12.2 Add integration tests: install/uninstall (--all, --skill, --plugin, --bundle), dry run, idempotency
- [ ] 12.3 Add integration tests: bundle create/delete/add/drop/swap, active bundle tracking
- [ ] 12.4 Add unit tests: URL parsing, source detection, config parsing, skill identity resolution, adapter logic
- [ ] 12.5 Verify all commands support --help, -h, help, --json, -n, -v, -q, --color
- [ ] 12.6 End-to-end smoke test: init → source add (local dir with skills) → target add → install --all → status → uninstall → cache clean
