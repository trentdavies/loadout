## MODIFIED Requirements

### Requirement: Skills follow Agent Skills specification
All skills in the registry SHALL conform to the Agent Skills specification (agentskills.io): a directory containing a `SKILL.md` file with YAML frontmatter (required: `name`, `description`; optional: `metadata.author`, `metadata.version`) and optional `scripts/`, `references/`, `assets/` directories. The `name` field SHALL match the directory name. Skill names MUST be kebab-case.

#### Scenario: Valid skill
- **WHEN** a skill directory contains `SKILL.md` with valid frontmatter including `name` and `description`
- **THEN** the skill SHALL be accepted into the registry

#### Scenario: Skill with optional metadata
- **WHEN** a skill's `SKILL.md` frontmatter includes `metadata.author` and `metadata.version`
- **THEN** the author and version SHALL be stored in the registry alongside the skill

#### Scenario: Invalid skill - missing frontmatter
- **WHEN** a skill directory contains a `SKILL.md` without `name` or `description` frontmatter
- **THEN** the CLI SHALL report a warning identifying the invalid skill and skip it during source registration

#### Scenario: Skill name mismatch
- **WHEN** a skill's `SKILL.md` frontmatter `name` does not match its directory name
- **THEN** the CLI SHALL report a warning about the mismatch and skip the skill

#### Scenario: Skill name not kebab-case
- **WHEN** a skill's `name` field contains characters other than lowercase letters, digits, and hyphens
- **THEN** the CLI SHALL report a warning and skip the skill
