## ADDED Requirements

### Requirement: Plugin as packaging unit
A plugin SHALL be the unit of packaging and distribution within a source. A source contains one or more plugins. A plugin contains one or more skills in a `skills/` subdirectory. A plugin MAY have a `.claude-plugin/plugin.json` manifest declaring its name, version, description, and author.

#### Scenario: Source with explicit plugins (marketplace)
- **WHEN** a source directory contains `.claude-plugin/marketplace.json` listing plugin entries
- **THEN** each plugin's `source` path SHALL be resolved and scanned for `.claude-plugin/plugin.json` metadata and `skills/` subdirectories

#### Scenario: Source with explicit plugin (single)
- **WHEN** a source directory contains `.claude-plugin/plugin.json`
- **THEN** the directory SHALL be recognized as a single plugin with metadata from `plugin.json`

#### Scenario: Source with implicit plugin
- **WHEN** a source directory has no `.claude-plugin` directory but contains skill directories
- **THEN** the source SHALL be wrapped in an implicit plugin named after the source

### Requirement: Plugin manifest format
A `.claude-plugin/plugin.json` SHALL be a JSON file with fields: `name` (required string), `version` (optional string), `description` (optional string), `author` (optional object with `name` field).

#### Scenario: Valid plugin.json
- **WHEN** a `.claude-plugin/plugin.json` contains name, version, and description
- **THEN** the plugin SHALL be loaded with all provided metadata

#### Scenario: plugin.json with missing name
- **WHEN** a `.claude-plugin/plugin.json` is missing the `name` field
- **THEN** the CLI SHALL report an error identifying the malformed plugin.json and its path

#### Scenario: Skills inferred from filesystem
- **WHEN** a plugin directory exists with or without `.claude-plugin/plugin.json`
- **THEN** skills SHALL be discovered by scanning the `skills/` subdirectory (or direct subdirectories) for directories containing `SKILL.md`

### Requirement: List plugins
The CLI SHALL support `skittle plugin list` to display all plugins in the registry, optionally filtered by `--source <name>`.

#### Scenario: List all plugins
- **WHEN** user runs `skittle plugin list`
- **THEN** the CLI SHALL display all plugins with their source, name, version, and skill count

#### Scenario: Filter by source
- **WHEN** user runs `skittle plugin list --source my-skills`
- **THEN** only plugins from the "my-skills" source SHALL be listed

### Requirement: Show plugin details
The CLI SHALL support `skittle plugin show <name>` to display a plugin's metadata and its skills.

#### Scenario: Show plugin
- **WHEN** user runs `skittle plugin show legal`
- **THEN** the CLI SHALL display the plugin name, version, source, description, and list all skills
