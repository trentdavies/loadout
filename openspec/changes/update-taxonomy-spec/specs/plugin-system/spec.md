## MODIFIED Requirements

### Requirement: Plugin as packaging unit
A plugin SHALL be the unit of packaging and distribution within a source. A source contains one or more plugins. A plugin contains one or more assets (skills in phase 1). A plugin MAY have a `plugin.toml` manifest or a `.claude-plugin` file declaring its metadata. When both exist, `plugin.toml` SHALL take precedence for name, version, and description; `.claude-plugin` SHALL supplement any fields not declared in `plugin.toml` (e.g., author).

#### Scenario: Source with explicit plugins
- **WHEN** a source directory contains subdirectories each with a `plugin.toml`
- **THEN** each subdirectory SHALL be recognized as a distinct plugin with metadata from its `plugin.toml`

#### Scenario: Source with .claude-plugin plugins
- **WHEN** a source directory contains subdirectories each with `.claude-plugin` but no `plugin.toml`
- **THEN** each subdirectory SHALL be recognized as a distinct plugin with metadata extracted from `.claude-plugin`

#### Scenario: Plugin with both plugin.toml and .claude-plugin
- **WHEN** a plugin directory contains both `plugin.toml` and `.claude-plugin`
- **THEN** `plugin.toml` SHALL be authoritative for name, version, and description
- **THEN** `.claude-plugin` SHALL fill in fields not present in `plugin.toml` (e.g., author)

#### Scenario: Source with implicit plugin
- **WHEN** a source directory has no `source.toml`, no `plugin.toml`, and no `.claude-plugin` but contains skill directories
- **THEN** the source SHALL be wrapped in an implicit plugin named after the source

## ADDED Requirements

### Requirement: .claude-plugin metadata extraction
When a `.claude-plugin` file is present, the system SHALL parse it defensively to extract plugin metadata. Missing or unexpected fields SHALL be treated as non-fatal (logged as warnings, not errors). The extracted fields SHALL include name, author, version, and description where available.

#### Scenario: Valid .claude-plugin file
- **WHEN** a directory contains a `.claude-plugin` file with name and version
- **THEN** the plugin SHALL use those values as metadata

#### Scenario: Malformed .claude-plugin file
- **WHEN** a directory contains a `.claude-plugin` file that cannot be parsed
- **THEN** the system SHALL log a warning and fall through to the next detection rule
- **THEN** the system SHALL NOT fail with an error

### Requirement: Implicit plugin naming from AgentSkill
When a source contains a single AgentSkill with no plugin manifest (no `plugin.toml`, no `.claude-plugin`), the implicit plugin name SHALL be the AgentSkill's name.

#### Scenario: Single AgentSkill without plugin
- **WHEN** a source resolves to a single AgentSkill directory named "code-review"
- **THEN** the implicit plugin name SHALL be "code-review"
- **THEN** the skill SHALL be accessible as `code-review/code-review`
