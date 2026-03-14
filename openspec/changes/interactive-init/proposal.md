## Why

`skittle init` currently creates directories and writes a default config — that's it. The first-run experience should be a guided wizard that gets you from zero to a working setup in one command: version-controlled data dir, detected targets, and popular skill sources — all with sensible defaults and minimal prompting.

## What Changes

- **`git init` the data dir**: After creating the skittle data directory, run `git init` at the top level so the config TOML, plugins, and local skills are version-controlled from the start. The `.gitignore` already excludes `external/` and `.skittle/` (cache/registry), so only user content is tracked.
- **Auto-detect targets**: Run the `target detect` logic with auto-add (no per-target prompting). User confirms with a single "Detect and add agent targets? [Y/n]" prompt.
- **Suggest popular marketplaces**: Present a curated list of known skill marketplaces (GitHub URLs) and let the user pick which to add. The list is stored in a maintainable location (a const array or a config file) so it can grow over time.
- **All prompts respect `--quiet`**: In quiet/non-interactive mode, use defaults (git init: yes, detect targets: yes, marketplaces: skip).
- **URL argument still works**: If `skittle init <url>` is provided, it adds that source and skips the marketplace prompt (existing behavior preserved).

## Capabilities

### New Capabilities
- `known-marketplaces`: A maintainable list of popular skill marketplace GitHub URLs with display names.

### Modified Capabilities
- `cli-framework`: `init` command gains interactive wizard flow (git init, target detect, marketplace selection).

## Impact

- `src/cli/mod.rs`: `Command::Init` handler gains the interactive wizard after directory creation.
- New const/static or config for the marketplace list (e.g., `src/marketplace.rs` or a const in `cli/mod.rs`).
- Reuses existing `target detect` logic (extract into a shared function if needed).
- `src/prompt.rs`: May need a multi-select prompt (pick multiple marketplaces from a list).
