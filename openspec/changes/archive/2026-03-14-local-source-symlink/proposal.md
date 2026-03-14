## Why

When adding a local directory source, skittle copies the entire tree into its cache. This means edits to the original source require a `skittle update` to propagate. For local development workflows — editing skills in a repo and testing them live — a symlink is faster and eliminates the sync step entirely. The copy behavior should remain available for cases where the user wants an immutable snapshot.

## What Changes

- **Structure-aware fetch mode**: The symlink/copy choice only applies to local *directory* sources (SinglePlugin, FlatSkills, Marketplace, SingleSkillDir). SingleFile sources are always copied — the copy is trivial and symlinking a single file into a cache directory is an awkward hybrid.
- `add` for local directory sources: After the existing interactive prompts, prompt whether to symlink or copy. Default: symlink.
- `fetch_local`: Add a `symlink` mode that creates a symlink from the cache path to the original source directory instead of copying.
- When symlink mode is used, `update` for that source skips re-fetch and only re-runs detect + normalize.
- `--symlink` / `--copy` flags bypass the prompt for scripted use. `--symlink` is the default when both are omitted in non-interactive mode. Flags are ignored for non-local sources and SingleFile sources.
- Config: persist the fetch mode (`symlink` or `copy`) in `SourceConfig` so `update` knows how to handle it.

## Capabilities

### New Capabilities
- (none — this extends existing fetch and interactive-prompts capabilities)

### Modified Capabilities
- `cli-framework`: `add` command gains `--symlink` / `--copy` flags and an interactive prompt for local directory sources.
- `interactive-prompts`: New prompt type for local source fetch mode (symlink vs copy).

## Impact

- `src/source/fetch.rs`: `fetch_local` gains a `symlink` parameter; symlink mode creates a directory symlink instead of copying. SingleFile path unchanged (always copies).
- `src/cli/mod.rs`: `Command::Add` gains `--symlink` / `--copy` flags; handler prompts only for local directory sources.
- `src/config/types.rs`: `SourceConfig` gains an optional `mode` field (`"symlink"` or `"copy"`).
- `src/source/fetch.rs`: detect symlinked sources during update and skip re-fetch.
- Existing tests that add local sources may need updating if they assert on copied files.
