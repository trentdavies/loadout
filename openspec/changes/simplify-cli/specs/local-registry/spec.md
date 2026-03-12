## MODIFIED Requirements

### Requirement: Registry index
The registry SHALL maintain a JSON index file (`registry.json`) that maps source, plugin, and skill identifiers to their cached filesystem paths. The index SHALL be updated when sources are added, removed, or updated.

#### Scenario: Registry reflects added source
- **WHEN** user runs `skittle add <url>` successfully
- **THEN** `registry.json` SHALL contain entries for the source, its plugins, and all discovered skills

#### Scenario: Registry reflects removed source
- **WHEN** user runs `skittle remove <name>`
- **THEN** all entries for that source, its plugins, and skills SHALL be removed from `registry.json`

### Requirement: Source content caching
Source content SHALL be cached in `~/.local/share/skittle/sources/<source-name>/`. The cached content SHALL mirror the source's directory structure (plugins, skills, and supporting files).

#### Scenario: Local source cached by copy
- **WHEN** a local filesystem source is added
- **THEN** its content SHALL be copied to the cache directory

#### Scenario: Git source cached by clone
- **WHEN** a git source is added
- **THEN** the repo SHALL be cloned (or shallow-cloned) to the cache directory

#### Scenario: Cache update replaces content
- **WHEN** user runs `skittle update <name>`
- **THEN** the cached content SHALL be replaced with the latest content from the source URL

### Requirement: Navigable by source, plugin, or skill
The registry SHALL support lookup by any level of the hierarchy: list all sources, list plugins within a source, list skills within a plugin, or look up a skill directly by `plugin/skill` identifier.

#### Scenario: Lookup by plugin/skill
- **WHEN** a skill is referenced as `openspec/explore`
- **THEN** the registry SHALL resolve it to the cached path and source metadata

#### Scenario: Ambiguous skill identity
- **WHEN** `openspec/explore` exists in multiple sources
- **THEN** the registry SHALL return an error listing all matching sources and requiring full `source:plugin/skill` qualification
