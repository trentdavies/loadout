## ADDED Requirements

### Requirement: XDG-compliant storage
The local registry SHALL store data at the XDG data directory (`~/.local/share/skittle/` on Linux/macOS). Configuration SHALL be stored at the XDG config directory (`~/.config/skittle/`).

#### Scenario: First run creates directories
- **WHEN** user runs any skittle command for the first time
- **THEN** the CLI SHALL create `~/.local/share/skittle/` and `~/.config/skittle/` if they do not exist

#### Scenario: XDG override
- **WHEN** `XDG_DATA_HOME` or `XDG_CONFIG_HOME` environment variables are set
- **THEN** the CLI SHALL use those paths instead of the defaults

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

### Requirement: Directory layout
The skittle data directory SHALL use this layout:
- `skittle.toml` — configuration (external sources, agents, bundles)
- `.claude-plugin/marketplace.json` — generated marketplace for managed plugins
- `.skittle/registry.json` — internal registry (gitignored)
- `plugins/` — managed plugins and skills (git tracked)
- `external/` — cached external source clones (gitignored)

#### Scenario: Fresh init
- **WHEN** user runs `skittle init`
- **THEN** the data directory SHALL be created with `skittle.toml`, `.skittle/`, `plugins/`, and `external/` directories
- **THEN** a `.gitignore` SHALL be created ignoring `external/` and `.skittle/`

#### Scenario: Legacy migration
- **WHEN** a `sources/` directory exists in the data directory
- **THEN** `skittle init` SHALL rename it to `external/`

### Requirement: Registry tracks provenance
The registry SHALL track provenance for each installed skill: which source, plugin, and skill it came from, and the relative path to the origin within the skittle data directory.

#### Scenario: Install records provenance
- **WHEN** a skill is installed from an external source to an agent
- **THEN** the registry SHALL record the source name, plugin name, skill name, and origin path (e.g., `external/anthropic-plugins/legal/skills/contract-review`)

#### Scenario: Install from managed plugin records provenance
- **WHEN** a skill is installed from `plugins/` to an agent
- **THEN** the registry SHALL record the origin path as `plugins/<plugin>/skills/<skill>`

### Requirement: Registry location
The registry SHALL be stored at `.skittle/registry.json` within the data directory. The `.skittle/` directory SHALL be gitignored.

#### Scenario: Registry at new path
- **WHEN** any skittle operation accesses the registry
- **THEN** it SHALL read from and write to `.skittle/registry.json`

### Requirement: External source cache location
External sources SHALL be cached in the `external/` directory instead of `sources/`.

#### Scenario: Add external source
- **WHEN** user runs `skittle add <git-url>`
- **THEN** the source SHALL be cloned into `external/<source-name>/`

#### Scenario: Cache dir for external sources
- **WHEN** any operation accesses the source cache
- **THEN** it SHALL use `data_dir()/external/` as the cache root
