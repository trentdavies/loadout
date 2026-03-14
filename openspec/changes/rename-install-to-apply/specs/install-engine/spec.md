## RENAMED Requirements

### Requirement: Install with explicit flags
FROM: Install with explicit flags
TO: Apply with explicit flags

### Requirement: Install all
FROM: Install all
TO: Apply all

### Requirement: Install skill
FROM: Install skill
TO: Apply skill

### Requirement: Install plugin
FROM: Install plugin
TO: Apply plugin

### Requirement: Install bundle
FROM: Install bundle
TO: Apply bundle

### Requirement: Dry run
FROM: Dry run
TO: Dry run

### Requirement: Idempotent install
FROM: Idempotent install
TO: Idempotent apply

## MODIFIED Requirements

### Requirement: Apply with explicit flags
The CLI SHALL support `skittle apply` with required flags to specify scope. Running `skittle apply` with no flags SHALL display the help text (not perform any action).

#### Scenario: Apply with no flags
- **WHEN** user runs `skittle apply`
- **THEN** the CLI SHALL display the apply command help and exit with a non-zero code

### Requirement: Apply all
The CLI SHALL support `skittle apply --all` to apply all configured skills to all auto-sync targets. Skills with status UNCHANGED SHALL be silently skipped. Skills with status CHANGED SHALL be subject to overwrite protection (see apply-conflict-detection spec).

#### Scenario: Apply all to auto targets
- **WHEN** user runs `skittle apply --all`
- **THEN** all skills referenced in the config SHALL be applied to all targets with sync mode `auto`, subject to conflict detection

#### Scenario: Apply all with target override
- **WHEN** user runs `skittle apply --all --target proj-claude`
- **THEN** all configured skills SHALL be applied to only the "proj-claude" target (regardless of its sync mode)

### Requirement: Apply skill
The CLI SHALL support `skittle apply --skill <plugin/skill>` to apply a specific skill. Optional `--target <name>` to limit to one target (defaults to all auto-sync targets).

#### Scenario: Apply specific skill
- **WHEN** user runs `skittle apply --skill openspec/explore`
- **THEN** "openspec/explore" SHALL be applied to all auto-sync targets, subject to conflict detection

#### Scenario: Apply skill to specific target
- **WHEN** user runs `skittle apply --skill openspec/explore --target claude-global`
- **THEN** "openspec/explore" SHALL be applied only to "claude-global"

#### Scenario: Apply nonexistent skill
- **WHEN** user runs `skittle apply --skill openspec/nonexistent`
- **THEN** the CLI SHALL exit with an error: "Skill 'openspec/nonexistent' not found in registry"

### Requirement: Apply plugin
The CLI SHALL support `skittle apply --plugin <name>` to apply all skills from a plugin. Optional `--target <name>`.

#### Scenario: Apply plugin
- **WHEN** user runs `skittle apply --plugin openspec`
- **THEN** all skills from the "openspec" plugin SHALL be applied to all auto-sync targets

### Requirement: Apply bundle
The CLI SHALL support `skittle apply --bundle <name>` to apply all skills from a bundle. Optional `--target <name>`.

#### Scenario: Apply bundle
- **WHEN** user runs `skittle apply --bundle work-dev`
- **THEN** all skills in the "work-dev" bundle SHALL be applied to all auto-sync targets
- **THEN** "work-dev" SHALL be recorded as the active bundle on those targets

### Requirement: Idempotent apply
Applying a skill that is already applied on a target with identical content SHALL be a no-op (no error, no file modification). The skill SHALL be counted as UNCHANGED in the summary.

#### Scenario: Reapply same version
- **WHEN** user runs `skittle apply --skill openspec/explore` and it's already applied with identical content
- **THEN** the skill SHALL be silently skipped and counted as unchanged
