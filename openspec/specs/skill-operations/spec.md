## ADDED Requirements

### Requirement: List skills
The CLI SHALL support `skittle skill list` to display all skills in the local registry. Optional filters: `--source <name>`, `--plugin <name>`.

#### Scenario: List all skills
- **WHEN** user runs `skittle skill list`
- **THEN** the CLI SHALL display all skills with their plugin, source, version, and installed targets

#### Scenario: Filter by plugin
- **WHEN** user runs `skittle skill list --plugin openspec`
- **THEN** only skills from the "openspec" plugin SHALL be listed

#### Scenario: Filter by source
- **WHEN** user runs `skittle skill list --source trent-skills`
- **THEN** only skills from the "trent-skills" source SHALL be listed

#### Scenario: No skills in registry
- **WHEN** user runs `skittle skill list` and no sources are registered
- **THEN** the CLI SHALL display "No skills found. Add a source with `skittle source add`"

### Requirement: Show skill details
The CLI SHALL support `skittle skill show <plugin/skill>` to display full details of a skill including its metadata (from SKILL.md frontmatter), which targets it's installed on, and its source/plugin context.

#### Scenario: Show skill by short identifier
- **WHEN** user runs `skittle skill show openspec/explore` and the identifier is unambiguous
- **THEN** the CLI SHALL display: name, description, version, license, compatibility, source, plugin, installed targets, and the skill directory contents

#### Scenario: Show skill by full identifier
- **WHEN** user runs `skittle skill show trent-skills:openspec/explore`
- **THEN** the CLI SHALL display the skill from the specified source

#### Scenario: Ambiguous skill identifier
- **WHEN** user runs `skittle skill show openspec/explore` and it exists in multiple sources
- **THEN** the CLI SHALL exit with an error listing matches and requiring `source:plugin/skill` disambiguation

### Requirement: Skills follow Agent Skills specification
All skills in the registry SHALL conform to the Agent Skills specification: a directory containing a `SKILL.md` file with YAML frontmatter (required: `name`, `description`) and optional `scripts/`, `references/`, `assets/` directories. The `name` field SHALL match the directory name.

#### Scenario: Valid skill
- **WHEN** a skill directory contains `SKILL.md` with valid frontmatter including `name` and `description`
- **THEN** the skill SHALL be accepted into the registry

#### Scenario: Invalid skill - missing frontmatter
- **WHEN** a skill directory contains a `SKILL.md` without `name` or `description` frontmatter
- **THEN** the CLI SHALL report a warning identifying the invalid skill and skip it during source registration

#### Scenario: Skill name mismatch
- **WHEN** a skill's `SKILL.md` frontmatter `name` does not match its directory name
- **THEN** the CLI SHALL report a warning about the mismatch and skip the skill
