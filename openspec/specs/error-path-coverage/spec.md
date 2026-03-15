## ADDED Requirements

### Requirement: Invalid input errors are tested
Operations receiving invalid input SHALL return descriptive errors rather than panicking.

#### Scenario: Add source with empty name
- **WHEN** a source is added with an empty string as the name
- **THEN** the operation returns an error

#### Scenario: Add agent with unknown agent type
- **WHEN** an agent is added with an agent type that has no adapter
- **THEN** the operation returns an error or warning about the unknown agent

#### Scenario: Install with no flags shows guidance
- **WHEN** install is called with no `--all`, `--skill`, `--plugin`, or `--bundle` flag
- **THEN** the operation returns an error indicating a flag is required

#### Scenario: Uninstall with no flags shows guidance
- **WHEN** uninstall is called with no targeting flags
- **THEN** the operation returns an error indicating a flag is required

#### Scenario: Bundle add with nonexistent skill identity
- **WHEN** a skill identity that doesn't match `plugin/skill` format is added to a bundle
- **THEN** the operation returns an error

### Requirement: Missing file errors are tested
Operations on nonexistent paths SHALL return clear errors.

#### Scenario: Load config from corrupted TOML
- **WHEN** `load_from()` is called on a file with `[invalid toml`
- **THEN** it returns a parse error

#### Scenario: Load registry from corrupted JSON
- **WHEN** `load_registry()` encounters `{broken json`
- **THEN** it returns a parse error

#### Scenario: Fetch from nonexistent local path
- **WHEN** `fetch()` is called with `/nonexistent/path`
- **THEN** it returns an error indicating the path does not exist

#### Scenario: Load manifest from nonexistent file
- **WHEN** `load_source_manifest()` is called with a nonexistent path
- **THEN** it returns a file-not-found error

#### Scenario: Detect on nonexistent path
- **WHEN** `detect()` is called on a path that does not exist
- **THEN** it returns an error

### Requirement: Dry-run prevents filesystem writes
Operations executed with `dry_run: true` SHALL not create, modify, or delete any files.

#### Scenario: Dry-run install writes nothing
- **WHEN** install is executed with dry-run enabled
- **THEN** no skill files are created in the agent directory

#### Scenario: Dry-run uninstall removes nothing
- **WHEN** uninstall is executed with dry-run enabled
- **THEN** installed skill files remain in the agent directory

#### Scenario: Dry-run source add modifies nothing
- **WHEN** source add is executed with dry-run enabled
- **THEN** the registry file is not modified

#### Scenario: Dry-run cache clean removes nothing
- **WHEN** cache clean is executed with dry-run enabled
- **THEN** cached source files remain in the cache directory

### Requirement: Edge cases in frontmatter parsing are tested
YAML frontmatter parsing SHALL handle various formatting edge cases.

#### Scenario: Frontmatter with extra whitespace
- **WHEN** a SKILL.md has frontmatter with leading/trailing whitespace around values
- **THEN** the parsed name and description are trimmed

#### Scenario: Frontmatter with quoted values
- **WHEN** a SKILL.md has `name: "my-skill"` with quotes
- **THEN** the parsed name is `my-skill` without quotes

#### Scenario: File with only frontmatter delimiters and no content
- **WHEN** a SKILL.md contains `---\n---` with nothing between
- **THEN** `has_skill_frontmatter()` returns `true` but `parse_skill_name()` returns `None`

#### Scenario: File with incomplete frontmatter
- **WHEN** a SKILL.md has an opening `---` but no closing `---`
- **THEN** `has_skill_frontmatter()` returns `false`

### Requirement: Manifest validation edge cases are tested
Manifest parsing SHALL validate field constraints.

#### Scenario: Source manifest with empty plugins list
- **WHEN** a `source.toml` has `plugins = []`
- **THEN** the manifest loads successfully with an empty plugins list

#### Scenario: Plugin manifest with optional fields
- **WHEN** a `plugin.toml` has `name` plus optional `version` and `description`
- **THEN** all fields are correctly parsed

#### Scenario: Manifest with unknown fields
- **WHEN** a TOML manifest contains fields not in the schema
- **THEN** parsing succeeds (unknown fields are ignored by serde default)
