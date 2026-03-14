## Why

Git source versioning is half-implemented. The `--ref` flag exists on `skittle add` and is stored in config, but the clone uses `--depth 1` without `--branch`, so the checkout silently fails for non-default refs. `skittle update` always resets to `origin/HEAD`, ignoring any pinned ref. There's no way to switch a source's ref after adding it, and no visibility into what version a source is on.

## What Changes

- Fix `fetch_git` to use `git clone --branch <ref> --depth 1` when a ref is provided
- Fix `update_git` to respect the stored ref: pull for branches, warn+skip for tags
- Add `--ref` flag to `skittle update` to switch a source's version in place
- Detect whether a ref is a tag or branch to determine update behavior
- Display the active ref in `skittle list` and `skittle status` output

## Capabilities

### New Capabilities
- `ref-detection`: Detect whether a git ref is a tag or branch to determine if a source is pinned (tag) or tracking (branch)

### Modified Capabilities
- `source-management`: Fix clone to honor ref, fix update to respect ref, add ref-switching via `skittle update --ref`

## Impact

- **Code**: `src/source/fetch.rs` (clone and update logic), `src/cli/mod.rs` (update command, list/status display)
- **Config**: No schema changes — `SourceConfig.ref` already exists
- **Behavior**: `skittle update` on a tagged source will now warn and skip instead of silently resetting to HEAD
