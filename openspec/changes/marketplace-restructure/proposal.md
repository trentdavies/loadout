## Why

Skittle's data directory should be a Claude marketplace — a valid, self-describing collection of plugins that any Claude-compatible tool can consume. Currently, `sources/` mixes external cached clones with any local content, there's no concept of "your plugins" vs "their plugins," and there's no way to get modified skills back from a target into version control. This restructure separates owned content from external dependencies, makes the skittle directory independently useful as a marketplace, and introduces `collect` for bidirectional skill flow.

## What Changes

- **Directory restructure**: `sources/` splits into `plugins/` (your authored/adopted skills, git tracked) and `external/` (cached clones of remote sources, gitignored). **BREAKING**: existing `sources/` directory moves to `external/`.
- **Marketplace generation**: `.claude-plugin/marketplace.json` is generated from `plugins/` only — it represents your curated collection, not external dependencies. Regenerated after any mutation (install, collect, source add/update).
- **Skittle internals move to `.skittle/`**: `registry.json` moves to `.skittle/registry.json`. The `.skittle/` directory is gitignored (machine-specific install state).
- **`skittle collect` command**: Copies modified skills from a target back to their source location. Registry tracks provenance (which source/plugin/skill, cache path) so collect knows where to put things.
- **`--adopt` flag on collect**: Graduates an external skill to `plugins/`, making it yours. Marketplace regenerated to include the adopted skill.
- **Untracked skill detection**: `skittle collect --target <name>` scans the target for skills not in the registry, offers to adopt them into `plugins/`.
- **`ref` pinning for git sources**: `skittle.toml` gains optional `ref` field for git sources (tag, branch, or commit SHA). Enables reproducible external source state.
- **`skittle.toml` scope change**: Only declares external sources, targets, and bundles. Managed plugins are self-describing via their `plugin.json` and filesystem presence in `plugins/`.

## Capabilities

### New Capabilities

- `skill-collect`: The `skittle collect` command — reverse of install. Copies skills from target back to cache (external) or plugins (local). Handles provenance lookup, `--adopt`, and untracked skill detection.
- `marketplace-generation`: Automatic generation of `.claude-plugin/marketplace.json` from the `plugins/` directory. The skittle data directory becomes a valid Claude marketplace.
- `source-pinning`: `ref` field in `skittle.toml` for git sources — pin to tag, branch, or commit.

### Modified Capabilities

- `local-registry`: Directory layout changes: `sources/` → `external/`, new `plugins/` dir, registry moves to `.skittle/`. Registry gains provenance tracking for installed skills.
- `install-engine`: Install records provenance in registry (source, plugin, skill, cache path). Used by collect to map skills back.
- `source-management`: `source add` for git/remote sources caches to `external/` instead of `sources/`. `ref` support for pinning.
- `cli-framework`: New `collect` command added to top-level command list.

## Impact

- **Directory layout**: `~/.local/share/skittle/` restructured. Existing users need migration (move `sources/` to `external/`).
- **Config**: `skittle.toml` gains `ref` field on sources. Loses any plugin declarations (plugins are filesystem-declared).
- **Registry**: `registry.json` moves to `.skittle/registry.json`, gains installed-skill provenance fields.
- **New command**: `skittle collect` with `--skill`, `--target`, `--adopt` flags.
- **Marketplace.json**: Generated file, committed to git. Any Claude-compatible tool can consume the skittle directory directly.
