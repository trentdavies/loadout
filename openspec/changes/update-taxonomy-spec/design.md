## Context

Skittle's source pipeline currently follows: `SourceUrl` (Local | Git) → `fetch` (copy or clone to cache) → `detect` (classify as SourceStructure) → `normalize` (produce RegisteredSource). This works for directory-based and git sources but can't handle zip archives, single remote files, or `.claude-plugin` metadata.

The proposal requires extending this pipeline to handle new input types while preserving the existing path for local/git sources. The CLI also needs simplified top-level commands and updated flag semantics (`--dry-run` stays global for additive commands, destructive commands use `--force`).

## Goals / Non-Goals

**Goals:**
- Extend `SourceUrl` to represent zip/archive and single-file sources
- Add zip unpacking to the fetch stage
- Teach detection about `.claude-plugin` files and archive contents
- Formalize the Source → ResolvedSource → Plugin → AgentSkill taxonomy in specs
- Add `skittle add`, `skittle list`, `skittle init [url]` shortcuts
- Restore `--dry-run` as a global flag (additive commands); keep `--force` for destructive commands

**Non-Goals:**
- Remote URL fetch for single files (phase 1 is local files and git only for non-archive sources)
- Plugin dependency resolution or version conflict handling
- Changing the on-disk registry format (`registry.json` structure stays the same)
- Supporting nested archives (zip within zip)

## Decisions

### 1. Extend SourceUrl with an Archive variant

Add `SourceUrl::Archive(PathBuf)` for `.zip` and `.skill` files. The parse function checks file extension before falling through to Local/Git classification.

**Why not a sub-type of Local?** Archives need distinct fetch behavior (unpack vs copy). Keeping them as a separate variant makes the fetch match arm explicit and avoids branching inside the Local handler.

### 2. Unpack archives in the fetch stage, not detect

Archives get unpacked into the cache directory during `fetch()`, producing a normal directory tree. Detection then runs on the unpacked contents with no knowledge it came from a zip.

**Alternative considered:** Detect could inspect zip contents directly. Rejected because it would duplicate filesystem traversal logic and every downstream stage would need archive awareness.

### 3. `.claude-plugin` as supplementary metadata, not replacement

When both `.claude-plugin` and `plugin.toml` exist, `plugin.toml` wins for name/version/description. `.claude-plugin` fills in gaps (e.g., author) but doesn't override explicit declarations.

**Why?** `plugin.toml` is skittle's own manifest and should be authoritative. `.claude-plugin` is a Claude Code artifact that may have different naming conventions. Supplement-not-replace avoids surprising behavior.

### 4. Detection priority order with new types

After fetch unpacks archives to a directory, detection runs the existing priority chain with one addition:

1. Single SKILL.md file
2. `source.toml` present → FullSource
3. `plugin.toml` present → SinglePlugin
4. `.claude-plugin` present → SinglePlugin (new — treated same as plugin.toml for structure, metadata extracted differently)
5. Subdirectories with SKILL.md → FlatSkills
6. Directory *is* an AgentSkill (has SKILL.md at root) → SingleSkillDir

No new `SourceStructure` variants needed. `.claude-plugin` maps to `SinglePlugin` and the manifest module handles metadata extraction.

### 5. Implicit naming rules

These happen in normalize, not detect:

- **No plugin, but AgentSkill exists:** Plugin name = skill name. This is already how SingleSkillDir works (wraps skill in implicit plugin).
- **No source name provided, plugin exists:** Source name = "local". Only applies to `skittle add <file>` type additions where no git origin exists.

### 6. CLI shortcut commands delegate to existing subcommands

`skittle add <url>` calls the same code path as `skittle source add <url>`. `skittle list [skills]` calls `skittle skill list`. `skittle init [url]` grabs the contents of the github url, and uses that as the full "skittle cache" located in ~/.local/share/skittle/.

These are defined as top-level `Command` variants that share implementation. No new modules.

### 7. `--dry-run` stays global, `--force` is per-command

`--dry-run` / `-n` returns as a global flag on the `Cli` struct. Additive commands (install, source add, source update, target add) check it. Destructive commands (uninstall, source remove, bundle delete, bundle swap, target remove, cache clean) ignore `--dry-run` and instead default to preview mode — they require `--force` to execute.

This means `skittle install -n --all` previews what would be installed, and `skittle uninstall --skill foo` previews what would be removed (no `-n` needed). `skittle uninstall --skill foo --force` actually removes it.

### 8. `zip` crate for archive support

Use the `zip` crate (well-maintained, pure Rust) for reading `.zip` and `.skill` files. No need for tar/gzip support in phase 1.

## Risks / Trade-offs

**[Zip bomb / large archives]** → Enforce a maximum unpacked size (e.g., 100MB) and maximum file count during extraction. Fail fast with a clear error.

**[`.claude-plugin` format may change]** → Parse defensively, treat missing/unexpected fields as non-fatal. Log warnings, don't fail. The format is not ours to control.

**[CLI shortcuts add surface area]** → Keep them as thin delegates. No unique logic in the shortcut paths. If the underlying command changes, the shortcut inherits the change.

**[`--dry-run` + `--force` interaction]** → If both are passed to a destructive command, `--dry-run` wins (preview only). Document this. Simpler than erroring on conflicting flags.
