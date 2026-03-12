## Why

The CLI has grown to 30+ commands across 8 subgroups. With source resolution handling URL→source→plugin→skill automatically, intermediate concepts (`source`, `plugin`, `skill`) don't need their own management commands. The `cache` subgroup is also unnecessary — `remove` already cleans the cache for a source. This change cuts the command surface roughly in half to make the CLI intuitive for new users.

## What Changes

- **BREAKING**: Remove `source` subgroup entirely — `add`, `remove`, `update` promoted to top-level commands; `source list` and `source show` removed (use `status` or `config show`)
- **BREAKING**: Remove `plugin` subgroup entirely — plugin info visible via `list` table columns
- **BREAKING**: Remove `skill` subgroup entirely — `list` is top-level; `skill show` folded into `list <name>`
- **BREAKING**: Remove `cache` subgroup entirely — no bulk cache clean; per-source cleanup happens via `remove`
- **BREAKING**: Remove `--source` and `--plugin` filter flags from skill listing
- `list` gains optional positional `name` argument for showing skill details
- `add` and `list` drop "shorthand" framing — they ARE the commands now

### Target CLI

```
skittle
├── init [URL]
├── add <url> [--name]
├── remove <name> [--force]
├── update [name]
├── list [name]
├── install / uninstall
├── status
├── bundle (create/delete/list/show/add/drop/swap)
├── target (add/remove/list/show/detect)
└── config (show/edit)
```

## Capabilities

### New Capabilities

(none — this change removes surface area, it does not add new capabilities)

### Modified Capabilities

- `cli-framework`: Command tree changes — remove source/plugin/skill/cache subgroups, add top-level remove/update
- `config-management`: Remove cache show/clean requirements
- `local-registry`: Update command references (source add → add, etc.)
- `test-suites`: Remove suite 04 (plugin system), update command names in other suite descriptions

### Removed Capabilities

- `source-management`: Absorbed into top-level add/remove/update — spec no longer needed
- `plugin-system`: No CLI surface — spec no longer needed
- `skill-operations`: Absorbed into top-level `list` — spec no longer needed

## Impact

- `src/cli/mod.rs` — All CLI definitions and command handlers (~600 lines removed)
- `tests/cli_flags.rs` — CLI parsing tests reference removed types
- `tests/integration_archive_and_shortcuts.rs` — References "shorthand" framing
- `tests/harness/setup.sh` — Uses `source add` in shared helper
- `tests/harness/suite/00_cli_framework.sh` — Tests removed subcommand help
- `tests/harness/suite/02_source_management.sh` — All tests use removed subcommands
- `tests/harness/suite/03_source_detection.sh` — Uses `source add`, `plugin list`
- `tests/harness/suite/04_plugin_system.sh` — Entire file deleted
- `tests/harness/suite/05_local_registry.sh` — Uses `source add`, `skill show`
- `tests/harness/suite/08_skill_operations.sh` — Uses `skill list/show`, filter flags
- `tests/harness/suite/11_end_to_end.sh` — Uses `source add/remove`, `cache clean`
- `openspec/specs/` — 3 specs deleted, 4 specs updated
