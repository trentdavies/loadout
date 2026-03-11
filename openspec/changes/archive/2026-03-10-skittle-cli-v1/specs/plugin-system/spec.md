## ADDED Requirements

### Requirement: Plugin as packaging unit
A plugin SHALL be the unit of packaging and distribution within a source. A source contains one or more plugins. A plugin contains one or more assets (skills in phase 1). A plugin MAY have a `plugin.toml` manifest declaring its name, version, description, and asset list.

#### Scenario: Source with explicit plugins
- **WHEN** a source directory contains subdirectories each with a `plugin.toml`
- **THEN** each subdirectory SHALL be recognized as a distinct plugin with metadata from its `plugin.toml`

#### Scenario: Source with implicit plugin
- **WHEN** a source directory has no `source.toml` and no `plugin.toml` but contains skill directories
- **THEN** the source SHALL be wrapped in an implicit plugin named after the source

### Requirement: Plugin manifest format
A `plugin.toml` SHALL support the fields: `name` (required string), `version` (optional string), `description` (optional string). The `[[asset]]` array SHALL declare assets with `name` (required), `type` (required, e.g. "skill"), and `version` (optional).

#### Scenario: Valid plugin.toml
- **WHEN** a `plugin.toml` contains name, version, and an asset list
- **THEN** the plugin SHALL be loaded with all declared metadata and assets

#### Scenario: Plugin.toml with missing name
- **WHEN** a `plugin.toml` is missing the `name` field
- **THEN** the CLI SHALL report an error identifying the malformed plugin.toml and its path

#### Scenario: Assets inferred from filesystem
- **WHEN** a `plugin.toml` exists but has no `[[asset]]` declarations
- **THEN** assets SHALL be inferred by scanning for skill directories (subdirectories containing `SKILL.md`)

### Requirement: List plugins
The CLI SHALL support `skittle plugin list` to display all plugins in the registry, optionally filtered by `--source <name>`.

#### Scenario: List all plugins
- **WHEN** user runs `skittle plugin list`
- **THEN** the CLI SHALL display all plugins with their source, name, version, and skill count

#### Scenario: Filter by source
- **WHEN** user runs `skittle plugin list --source my-skills`
- **THEN** only plugins from the "my-skills" source SHALL be listed

### Requirement: Show plugin details
The CLI SHALL support `skittle plugin show <name>` to display a plugin's metadata and its assets grouped by type.

#### Scenario: Show plugin
- **WHEN** user runs `skittle plugin show openspec`
- **THEN** the CLI SHALL display the plugin name, version, source, description, and list all assets with their type and version
