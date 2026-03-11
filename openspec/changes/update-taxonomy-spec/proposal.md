## Why

Skittle's current specs were written around a narrow model: sources are either local directories or git repos, and detection logic is tightly coupled to that assumption. Notes.md introduces a broader taxonomy (zip files, `.skill` archives, single files, `.claude-plugin` metadata) and simplified top-level commands (`skittle add`, `skittle init [url]`) that the current specs don't account for. The specs need to be updated before building these capabilities, since the source resolution pipeline, plugin metadata, and CLI surface all change.

## What Changes

- **Richer source resolution**: Sources can now resolve to zip files (including `.skill` extension), single SKILL.md files, directories containing plugins, or directories that *are* a plugin. The current detection spec only handles directory and git sources. **BREAKING**: `SourceUrl` parsing and `detect` must handle new input types.
- **Zip/archive support**: `.skill` and `.zip` files are valid sources. They may contain a plugin, an AgentSkill directory, or the *contents* of an AgentSkill directory (loose files). The fetch and detect pipeline must unpack and classify these.
- **`.claude-plugin` integration**: If a plugin directory contains a `.claude-plugin` file, its name/author/version should be used as plugin metadata, supplementing or replacing `plugin.toml`.
- **Formal taxonomy**: Codify the Source → ResolvedSource → Plugin → AgentSkill hierarchy as the canonical model. Current specs describe pieces of this but don't name or formalize the resolution pipeline.
- **Simplified top-level commands**: `skittle init [github|path]` combines initialization with first source addition. `skittle add <source>` is a shorthand for `skittle source add`. `skittle list [plugins]` is a shorthand for listing skills or plugins.
- **AgentSkill spec alignment**: Skills must conform to the agentskills.io specification. Author and version are optional YAML frontmatter fields (`metadata.author`, `metadata.version`). Skill names must be kebab-case.
- **Implicit plugin/source naming**: When no plugin is defined but an AgentSkill exists, the plugin name defaults to the skill name. When no source is defined but a plugin exists, the source defaults to "local".
- **CLI flag changes already landed**: `--dry-run` removed as global flag; destructive commands default to dry-run and require `--force`. `--color` removed (auto-detected). These need spec updates.
  - We need --dry-run available for non-destructive (additive) commands. 

## Capabilities

### New Capabilities

- `source-resolution`: Formal taxonomy and pipeline for resolving a source URL/path into a classified ResolvedSource (directory-of-plugins, single-plugin, agent-skill-directory, zip-archive, single-file). This is the conceptual model that source-detection implements.
- `archive-sources`: Support for `.zip` and `.skill` archive files as source inputs — fetch, unpack, and classify their contents.
- `cli-shortcuts`: Top-level shorthand commands (`skittle add`, `skittle list`, `skittle init [url]`) that delegate to existing subcommands.

### Modified Capabilities

- `source-detection`: Detection must handle zip/archive inputs, `.claude-plugin` metadata, and the formal ResolvedSource classification.
- `source-management`: `source add` must accept source URLs. zip files and single SKILL.md files are now valid source URLs. 
- `plugin-system`: Plugins can derive metadata from `.claude-plugin` in addition to `plugin.toml`. Implicit plugin naming when only an AgentSkill exists.
- `skill-operations`: Formalize agentskills.io alignment — optional `metadata.author` and `metadata.version` in frontmatter, kebab-case name requirement.
- `cli-framework`: Keep `--dry-run` in global flags. Document `--force` on destructive commands. Add top-level shorthand commands.
- `install-engine`: Remove `--dry-run` references. Destructive operations (`uninstall`) default to preview and require `--force`.

## Impact

- **Source pipeline** (`src/source/`): `url.rs`, `fetch.rs`, `detect.rs`, `normalize.rs` all need updates for new source types.
- **CLI** (`src/cli/mod.rs`): New top-level commands, flag changes already in code but specs need to catch up.
- **Plugin metadata** (`src/source/manifest.rs`): `.claude-plugin` parsing alongside `plugin.toml`.
- **Dependencies**: May need a zip/archive crate (e.g., `zip`).
- **Tests**: Detection and fetch tests need fixtures for zip files, single files, and `.claude-plugin` directories.
