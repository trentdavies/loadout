## ADDED Requirements

### Requirement: Progressive source structure detection
When a source URL is resolved to a local path, the CLI SHALL detect its structure using the following algorithm in order:

1. If path is a file and looks like a skill (contains YAML frontmatter with `name` and `description` fields) → single skill
2. If directory contains `source.toml` → full multi-plugin source
3. If directory contains `plugin.toml` → single plugin
4. If directory contains subdirectories with `SKILL.md` files → flat plugin (inferred)
5. If directory contains `SKILL.md` directly → single skill
6. Otherwise → error: cannot determine source structure

#### Scenario: Single SKILL.md file
- **WHEN** user runs `skittle source add ~/my-skill.md`
- **THEN** the CLI SHALL create an implicit source and implicit plugin both named from the file (e.g., "my-skill")
- **THEN** the single skill SHALL be accessible as `my-skill/my-skill`

#### Scenario: Full source with source.toml
- **WHEN** user runs `skittle source add ~/marketplace` and the directory contains `source.toml`
- **THEN** the CLI SHALL read `source.toml` for source metadata
- **THEN** the CLI SHALL discover plugins from subdirectories listed in `source.toml` or by scanning for `plugin.toml` files

#### Scenario: Single plugin with plugin.toml
- **WHEN** user runs `skittle source add ~/openspec-plugin` and the directory contains `plugin.toml` but no `source.toml`
- **THEN** the CLI SHALL wrap the plugin in an implicit source named from the directory
- **THEN** the plugin SHALL be accessible by its name from `plugin.toml`

#### Scenario: Flat directory with skill subdirectories
- **WHEN** user runs `skittle source add ~/my-skills/` and the directory has no TOML manifests but contains subdirectories with `SKILL.md` files
- **THEN** the CLI SHALL infer a flat plugin named after the directory
- **THEN** each subdirectory with a `SKILL.md` SHALL be registered as a skill

#### Scenario: Unrecognizable directory
- **WHEN** user runs `skittle source add ~/random-dir` and none of the detection rules match
- **THEN** the CLI SHALL exit with an error explaining what structures are expected

### Requirement: Source manifest format
A `source.toml` SHALL support the fields: `name` (required string), `version` (optional string), `description` (optional string), `plugins` (optional array of directory names — if omitted, plugins are auto-discovered from subdirectories containing `plugin.toml`).

#### Scenario: Explicit plugin list
- **WHEN** `source.toml` contains `plugins = ["openspec", "writing"]`
- **THEN** only those subdirectories SHALL be scanned for plugins

#### Scenario: Auto-discovered plugins
- **WHEN** `source.toml` has no `plugins` field
- **THEN** all subdirectories containing `plugin.toml` SHALL be discovered as plugins

### Requirement: Implicit naming from URL
When no `--name` is provided and no manifest declares a name, the source name SHALL be derived from the URL: directory name for local paths, repository name for git URLs, filename (without extension) for single files.

#### Scenario: Name from directory
- **WHEN** user runs `skittle source add ~/dev/agent-skills/` with no `--name` and no `source.toml`
- **THEN** the source SHALL be named "agent-skills"

#### Scenario: Name from git URL
- **WHEN** user runs `skittle source add https://github.com/org/my-tools.git` with no `--name`
- **THEN** the source SHALL be named "my-tools"
