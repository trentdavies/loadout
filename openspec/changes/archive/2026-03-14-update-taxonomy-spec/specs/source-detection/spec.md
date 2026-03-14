## MODIFIED Requirements

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
