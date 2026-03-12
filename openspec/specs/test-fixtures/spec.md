## ADDED Requirements

### Requirement: Single-file skill fixture
The fixtures SHALL include a `single-skill/` directory containing one valid `SKILL.md` with proper YAML frontmatter (`name: single-skill`, `description` set). This tests the simplest source type — a lone skill file.

#### Scenario: Single-skill fixture is valid
- **WHEN** `skittle add` is pointed at `fixtures/single-skill/SKILL.md`
- **THEN** it SHALL be detected as a single-file source and register as source "single-skill" with one implicit plugin and one skill

### Requirement: Flat directory skill fixture
The fixtures SHALL include a `flat-skills/` directory containing two subdirectories (`explore/` and `apply/`), each with a valid `SKILL.md`. No `plugin.toml` or `source.toml`. This tests inferred flat plugin detection.

#### Scenario: Flat-skills fixture is valid
- **WHEN** `skittle add` is pointed at `fixtures/flat-skills/`
- **THEN** it SHALL be detected as a flat plugin with 2 skills: `flat-skills/explore` and `flat-skills/apply`

### Requirement: Plugin source fixture
The fixtures SHALL include a `plugin-source/` directory with a `plugin.toml` (name: "test-plugin", version: "0.1.0") and a `skills/` subdirectory containing 3 skills: `explore`, `apply`, and `verify`. The `apply` skill SHALL include a `scripts/` subdirectory to test supporting directory copying.

#### Scenario: Plugin-source fixture has valid plugin.toml
- **WHEN** `skittle add` is pointed at `fixtures/plugin-source/`
- **THEN** it SHALL be detected as a single-plugin source with name "test-plugin" and 3 skills

#### Scenario: Plugin-source apply skill has scripts
- **WHEN** the "test-plugin/apply" skill is installed to a target
- **THEN** the `scripts/` directory SHALL be copied alongside the `SKILL.md`

### Requirement: Full multi-plugin source fixture
The fixtures SHALL include a `full-source/` directory with a `source.toml` (name: "test-source", version: "1.0.0") and two plugin subdirectories: `test-plugin-a/` (2 skills: `skill-one`, `skill-two`) and `test-plugin-b/` (1 skill: `skill-three`). Each plugin SHALL have a `plugin.toml`.

#### Scenario: Full-source fixture is valid
- **WHEN** `skittle add` is pointed at `fixtures/full-source/`
- **THEN** it SHALL be detected as a full source with 2 plugins and 3 total skills

#### Scenario: Full-source skills are addressable
- **WHEN** the source is registered
- **THEN** skills SHALL be addressable as `test-plugin-a/skill-one`, `test-plugin-a/skill-two`, and `test-plugin-b/skill-three`

### Requirement: All fixture skills follow Agent Skills spec
Every `SKILL.md` in the fixtures SHALL have valid YAML frontmatter with `name` (matching its directory name) and `description` (non-empty). Names SHALL use lowercase and hyphens only.

#### Scenario: Fixture skill frontmatter is valid
- **WHEN** any fixture `SKILL.md` is parsed
- **THEN** it SHALL have a `name` field matching its parent directory and a non-empty `description` field

### Requirement: Invalid fixture for negative testing
The fixtures SHALL include an `invalid/` directory containing: a `no-frontmatter/SKILL.md` (markdown with no YAML frontmatter), a `bad-name/SKILL.md` (frontmatter `name` does not match directory), and an empty directory `empty-dir/`. These are used to test error handling.

#### Scenario: No-frontmatter fixture triggers warning
- **WHEN** `skittle add` is pointed at a source containing `no-frontmatter/`
- **THEN** the CLI SHALL warn about the invalid skill and skip it

#### Scenario: Bad-name fixture triggers warning
- **WHEN** `skittle add` encounters a skill where `name` doesn't match the directory
- **THEN** the CLI SHALL warn about the name mismatch and skip the skill

#### Scenario: Empty directory is not detected as a skill
- **WHEN** `skittle add` scans a directory containing `empty-dir/`
- **THEN** `empty-dir` SHALL NOT be registered as a skill
