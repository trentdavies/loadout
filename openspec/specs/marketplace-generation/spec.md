## ADDED Requirements

### Requirement: Generate marketplace.json from plugins directory
The system SHALL generate `.claude-plugin/marketplace.json` by scanning the `plugins/` directory. Only managed plugins SHALL appear in the marketplace — external sources SHALL NOT be included.

#### Scenario: Plugins directory with multiple plugins
- **WHEN** `plugins/` contains `my-tools/` and `team-utils/` each with `.claude-plugin/plugin.json`
- **THEN** marketplace.json SHALL list both plugins with `source` paths relative to the skittle data dir (e.g., `"./plugins/my-tools"`)

#### Scenario: Plugin without plugin.json
- **WHEN** `plugins/` contains a directory with `skills/` but no `.claude-plugin/plugin.json`
- **THEN** the plugin SHALL still appear in marketplace.json with name derived from the directory

#### Scenario: Empty plugins directory
- **WHEN** `plugins/` is empty or does not exist
- **THEN** marketplace.json SHALL contain an empty `plugins` array

### Requirement: Marketplace regenerated on mutation
The marketplace.json SHALL be regenerated after any operation that changes `plugins/`: `collect --adopt`, plugin creation, or skill adoption. It SHALL NOT be regenerated on operations that only affect `external/` or agents.

#### Scenario: After adopt
- **WHEN** user runs `skittle collect --skill foo --agent claude --adopt`
- **THEN** marketplace.json SHALL be regenerated to include the newly adopted plugin

### Requirement: Marketplace is valid Claude format
The generated marketplace.json SHALL conform to the Claude marketplace format: `name` (string), `owner` (optional object with `name`), `plugins` (array of plugin entries with `name`, `source`, and optional `description`).

#### Scenario: Valid marketplace consumed externally
- **WHEN** the skittle data directory is pointed to by another Claude-compatible tool
- **THEN** the tool SHALL be able to read marketplace.json and discover all managed plugins
