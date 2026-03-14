## ADDED Requirements

### Requirement: Skill status detection
The adapter SHALL compare a source skill directory against its installed counterpart at the target and return one of three statuses: NEW (not present at target), UNCHANGED (all files match byte-for-byte), or CHANGED (at least one file differs in content, is missing at target, or exists only at source).

#### Scenario: Skill not present at target
- **WHEN** the source skill directory has no corresponding directory at the target
- **THEN** the adapter SHALL return status NEW

#### Scenario: Skill identical at target
- **WHEN** every file in the source skill directory has a byte-identical counterpart at the target, and no extra files exist at either location
- **THEN** the adapter SHALL return status UNCHANGED

#### Scenario: Skill file content differs
- **WHEN** at least one file in the source skill directory has different content than its counterpart at the target
- **THEN** the adapter SHALL return status CHANGED

#### Scenario: Source has file not present at target
- **WHEN** the source skill directory contains a file that does not exist at the target
- **THEN** the adapter SHALL return status CHANGED

#### Scenario: Target has extra file not in source
- **WHEN** the target skill directory contains a file that does not exist in the source
- **THEN** the adapter SHALL return status CHANGED

### Requirement: Per-skill granularity
Conflict detection SHALL operate at the skill directory level. If any file within a skill directory differs, the entire skill SHALL be flagged as CHANGED. Individual file-level conflict reporting is not required.

#### Scenario: Multiple files differ
- **WHEN** SKILL.md and scripts/run.sh both differ between source and target
- **THEN** the adapter SHALL return a single CHANGED status for the skill (not two separate conflicts)

### Requirement: Default overwrite protection
When applying skills, if any skill has status CHANGED and neither `--force` nor `--interactive` is specified, the apply operation SHALL refuse to proceed and exit with an error listing the conflicting skills and suggesting `--force` or `-i`.

#### Scenario: Conflict without force or interactive
- **WHEN** user runs `skittle apply --all` and skill "openspec/explore" has status CHANGED
- **THEN** the CLI SHALL exit with a non-zero code and display: the conflicting skill name(s), and a suggestion to use `--force` or `-i` to resolve

#### Scenario: No conflicts proceed normally
- **WHEN** user runs `skittle apply --all` and all skills are NEW or UNCHANGED
- **THEN** the CLI SHALL apply all NEW skills and skip all UNCHANGED skills without error

### Requirement: Force flag bypasses protection
When `--force` or `-f` is specified, all CHANGED skills SHALL be overwritten without prompting.

#### Scenario: Force overwrite
- **WHEN** user runs `skittle apply --all --force` and skill "openspec/explore" has status CHANGED
- **THEN** the skill SHALL be overwritten at the target without prompting
