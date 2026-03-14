## Context

`fetch_local` currently does a recursive directory copy (skipping `.git`). For local development, this means every edit requires `skittle update` to resync. A symlink from the cache to the original source eliminates this friction entirely.

The choice is structure-aware: SingleFile sources always copy (trivial cost, awkward symlink semantics). Directory sources (SinglePlugin, FlatSkills, Marketplace, SingleSkillDir) prompt for symlink vs copy.

## Goals / Non-Goals

**Goals:**
- Symlink local directory sources by default for live-edit workflows
- Copy mode remains available for snapshot behavior
- SingleFile sources always copy — no prompt, no flags
- Persist the mode in config so `update` behaves correctly

**Non-Goals:**
- Changing fetch behavior for git or archive sources
- Watching for filesystem changes or auto-reloading
- Partial symlink (symlink some files, copy others)

## Decisions

### 1. Structure-aware: files copy, directories link

The symlink/copy choice only fires for local directory sources. SingleFile is always copied because:
- Copy cost is negligible (one file)
- Symlinking a file inside a real cache directory is a hybrid that complicates cache assumptions
- No meaningful workflow benefit

Detection: `source_url.is_local()` AND `source_path.is_dir()` determines eligibility.

### 2. Symlink the cache directory itself

For directory sources, create a single symlink at the cache path pointing to the original source directory. This is simpler than symlinking individual files and naturally handles subdirectory structure changes.

`std::os::unix::fs::symlink(source_path, cache_path)` — the cache_path must not already exist.

### 3. Persist mode in SourceConfig

Add an optional `mode` field to `SourceConfig` (values: `"symlink"` or `"copy"`, default omitted = `"copy"` for backward compatibility). This lets `update` know whether to re-fetch or skip.

### 4. Prompt placement: after source name confirmation, before fetch

The symlink/copy prompt fires only for eligible local sources (directories), after the source name is confirmed but before `fetch` is called. This way `fetch` receives the mode and acts accordingly.

### 5. Flags: `--symlink` and `--copy`

Two mutually exclusive boolean flags (clap `conflicts_with`). When neither is passed and the source is an eligible local directory, prompt. When neither is passed and non-interactive, default to symlink. For non-local sources or SingleFile, both flags are silently ignored.

### 6. Update behavior for symlinked sources

When `update` encounters a source with `mode: "symlink"`, it skips re-fetch and just re-runs detect + normalize to pick up structural changes (new skills, renamed directories). The cache symlink already points to the live source.

## Risks / Trade-offs

- **Symlink breakage**: If the original source is moved or deleted, the symlink breaks. Mitigation: `update` and `status` can detect broken symlinks and warn.
- **Cross-device symlinks**: Symlinks work within the same filesystem. Mitigation: fall back to copy with a warning if symlink creation fails.
