## ADDED Requirements

### Requirement: Formal source taxonomy
The system SHALL use a four-level taxonomy for all source content: Source → ResolvedSource → Plugin → AgentSkill. Every source input SHALL be classified into exactly one ResolvedSource type before further processing.

#### Scenario: Local directory with plugins
- **WHEN** a source URL resolves to a directory containing plugin subdirectories
- **THEN** the ResolvedSource SHALL be classified as "directory-of-plugins"

#### Scenario: Local directory that is a plugin
- **WHEN** a source URL resolves to a directory containing `plugin.toml` or `.claude-plugin` but no `source.toml`
- **THEN** the ResolvedSource SHALL be classified as "single-plugin"

#### Scenario: AgentSkill directory
- **WHEN** a source URL resolves to a directory containing `SKILL.md` at its root
- **THEN** the ResolvedSource SHALL be classified as "agent-skill-directory"

#### Scenario: Archive file
- **WHEN** a source URL resolves to a `.zip` or `.skill` file
- **THEN** the ResolvedSource SHALL be classified as "zip-archive"

#### Scenario: Single SKILL.md file
- **WHEN** a source URL resolves to a single file with YAML frontmatter containing `name` and `description`
- **THEN** the ResolvedSource SHALL be classified as "single-file"

### Requirement: ResolvedSource classification types
The system SHALL support exactly these ResolvedSource types: `directory-of-plugins`, `single-plugin`, `agent-skill-directory`, `zip-archive`, `single-file`. Each type SHALL map to the existing `SourceStructure` variants after fetch and detection.

#### Scenario: Classification to SourceStructure mapping
- **WHEN** a ResolvedSource is classified
- **THEN** `directory-of-plugins` SHALL map to `FullSource`
- **THEN** `single-plugin` SHALL map to `SinglePlugin`
- **THEN** `agent-skill-directory` SHALL map to `SingleSkillDir`
- **THEN** `zip-archive` SHALL map to whichever SourceStructure the unpacked contents match
- **THEN** `single-file` SHALL map to `SingleFile`

### Requirement: Pipeline order
The source pipeline SHALL execute in this order: parse URL → fetch (copy, clone, or unpack) → detect (classify SourceStructure) → normalize (produce RegisteredSource). No stage SHALL depend on knowledge of stages that follow it.

#### Scenario: Archive goes through full pipeline
- **WHEN** a `.skill` file is added as a source
- **THEN** fetch SHALL unpack it to the cache directory
- **THEN** detect SHALL run on the unpacked directory with no knowledge it came from an archive
- **THEN** normalize SHALL produce a RegisteredSource from the detected structure

### Requirement: Implicit naming defaults
When normalizing a source, the system SHALL apply these naming defaults: if no plugin is defined but an AgentSkill exists, the plugin name SHALL default to the skill name. If no source name is provided and no git origin exists, the source name SHALL default to "local".

#### Scenario: AgentSkill without plugin
- **WHEN** a source contains a single AgentSkill named "my-tool" with no plugin manifest
- **THEN** the implicit plugin name SHALL be "my-tool"

#### Scenario: Local file with no source name
- **WHEN** user runs `skittle add ~/my-skill.md` with no `--name` flag
- **THEN** the source name SHALL be derived from the filename ("my-skill")
