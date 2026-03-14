# Merge Notes: `command-clean`

## What changed

Unified all skill identity display to use a single `source:plugin/skill` format throughout CLI output, replacing scattered source/plugin/skill columns and status lines.

## Files touched

### `src/output/mod.rs`
- Added `format_identity()` — color-coded identity string (cyan `:` green `/` bold)
- Added `plain_identity()` — plain text variant for JSON mode

### `src/cli/mod.rs`
- Added `use colored::Colorize` import
- **List command**: replaced 3-column table with 2-column (Identity, Description), custom padding logic to handle ANSI escape codes in colored strings
- **List detail**: collapsed Skill/Plugin/Source status lines into single Identity line
- **List JSON**: added `"identity"` field to both single-skill and all-skills output
- **Install dry run**: skill name → `format_identity()`
- **Uninstall preview**: looks up provenance for colored identity
- **Collect tracked listing**: stores full `InstalledSkill` instead of origin string, displays colored identity
- **Collect/adopt (single)**: uses provenance for identity in success message
- **Collect/adopt (bulk)**: untracked skills show as `local:local/skill`

## Conflict-prone areas

- **List command (~line 425–496)**: Heavy rewrite of the table rendering. Any change to list output on `main` will conflict.
- **Uninstall (~line 657–665)**: Added provenance lookup block. Conflicts if uninstall output changed on `main`.
- **Collect (~line 696–760)**: Changed data flow for `tracked` vector (now stores `InstalledSkill` instead of `String`). Conflicts if collect logic changed on `main`.

## Merge strategy

If conflicts arise in the table-rendering section of `list`, take this branch's version — the old 3-column table code is fully replaced. For other conflicts, the key invariant is: anywhere a skill identity is displayed, it should go through `format_identity()` (colored) or `plain_identity()` (JSON).

## Verification after merge

```sh
cargo build          # no warnings
cargo test           # all pass
skittle list         # colored identity + description table
skittle list --json  # identity field present
NO_COLOR=1 skittle list  # plain text, no ANSI codes
```
