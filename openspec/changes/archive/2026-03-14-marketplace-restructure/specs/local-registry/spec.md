## MODIFIED Requirements

### Requirement: Directory layout
The skittle data directory SHALL use this layout:
- `skittle.toml` — configuration (external sources, targets, bundles)
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
- **WHEN** a skill is installed from an external source to a target
- **THEN** the registry SHALL record the source name, plugin name, skill name, and origin path (e.g., `external/anthropic-plugins/legal/skills/contract-review`)

#### Scenario: Install from managed plugin records provenance
- **WHEN** a skill is installed from `plugins/` to a target
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
