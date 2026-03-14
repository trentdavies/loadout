## ADDED Requirements

### Requirement: Progressive source structure detection
When a source URL is resolved to a local path, the CLI SHALL detect its structure using the following algorithm in order:

1. If path is a file and looks like a skill (contains YAML frontmatter with `name` and `description` fields) → single skill
2. If directory contains `source.toml` → full multi-plugin source
3. If directory contains `plugin.toml` → single plugin
4. If directory contains `.claude-plugin` → single plugin (metadata extracted from `.claude-plugin` format)
5. If directory contains subdirectories with `SKILL.md` files → flat plugin (inferred)
6. If directory contains `SKILL.md` directly → single skill
7. Otherwise → error: cannot determine source structure

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

#### Scenario: Single plugin with .claude-plugin
- **WHEN** user runs `skittle source add ~/my-plugin` and the directory contains `.claude-plugin` but no `source.toml` or `plugin.toml`
- **THEN** the CLI SHALL detect it as a single plugin
- **THEN** plugin metadata (name, author, version) SHALL be extracted from `.claude-plugin`

#### Scenario: Flat directory with skill subdirectories
- **WHEN** user runs `skittle source add ~/my-skills/` and the directory has no TOML manifests and no `.claude-plugin` but contains subdirectories with `SKILL.md` files
- **THEN** the CLI SHALL infer a flat plugin named after the directory
- **THEN** each subdirectory with a `SKILL.md` SHALL be registered as a skill

#### Scenario: Unpacked archive directory
- **WHEN** an archive has been unpacked to the cache and detection runs on the result
- **THEN** detection SHALL apply the same algorithm as any other directory with no special archive handling

#### Scenario: Unrecognizable directory
- **WHEN** user runs `skittle source add ~/random-dir` and none of the detection rules match
- **THEN** the CLI SHALL exit with an error explaining what structures are expected

### Requirement: Marketplace manifest format
A `.claude-plugin/marketplace.json` SHALL be a JSON file with fields: `name` (required string), `owner` (optional object with `name`), `plugins` (required array of plugin entries). Each plugin entry SHALL have `name` (required), `source` (required, relative path to plugin directory), `description` (optional), and `author` (optional object with `name`).

#### Scenario: Explicit plugin list in marketplace
- **WHEN** `marketplace.json` contains a `plugins` array with entries like `{"name": "legal", "source": "./legal"}`
- **THEN** those directories SHALL be resolved relative to the marketplace root and scanned as plugins

#### Scenario: Missing plugin directory
- **WHEN** a marketplace plugin entry references a `source` path that does not exist
- **THEN** the CLI SHALL warn and skip that plugin without failing

### Requirement: Plugin manifest format
A `.claude-plugin/plugin.json` SHALL be a JSON file with fields: `name` (required string), `version` (optional string), `description` (optional string), `author` (optional object with `name`).

#### Scenario: Plugin with full metadata
- **WHEN** a plugin directory contains `.claude-plugin/plugin.json` with name, version, description, and author
- **THEN** the plugin SHALL be registered with all provided metadata

#### Scenario: Plugin without manifest
- **WHEN** a plugin directory has no `.claude-plugin/plugin.json`
- **THEN** the plugin name SHALL be derived from the directory name

### Requirement: Implicit naming from URL
When no `--name` is provided and no manifest declares a name, the source name SHALL be derived from the URL: directory name for local paths, repository name for git URLs, filename (without extension) for single files.

#### Scenario: Name from directory
- **WHEN** user runs `skittle add ~/dev/agent-skills/` with no `--name`
- **THEN** the source SHALL be named "agent-skills"

#### Scenario: Name from git URL
- **WHEN** user runs `skittle add https://github.com/org/my-tools.git` with no `--name`
- **THEN** the source SHALL be named "my-tools"
