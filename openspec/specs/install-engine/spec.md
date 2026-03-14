## ADDED Requirements

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

### Requirement: Dry run
Additive operations (install) SHALL support `-n` / `--dry-run` which displays what would change without making modifications. Destructive operations (uninstall) SHALL default to preview mode and require `--force` to execute.

#### Scenario: Dry run install
- **WHEN** user runs `skittle install --all -n`
- **THEN** the CLI SHALL display a list of skills that would be installed and to which targets, without writing any files

#### Scenario: Uninstall preview (default)
- **WHEN** user runs `skittle uninstall --skill openspec/explore` without `--force`
- **THEN** the CLI SHALL display what would be uninstalled without removing any files
- **THEN** the CLI SHALL print "Use --force to uninstall"

#### Scenario: Uninstall with --force
- **WHEN** user runs `skittle uninstall --skill openspec/explore --force`
- **THEN** "openspec/explore" SHALL be removed from all targets where it's installed

### Requirement: Idempotent apply
Applying a skill that is already applied on a target with identical content SHALL be a no-op (no error, no file modification). The skill SHALL be counted as UNCHANGED in the summary.

#### Scenario: Reapply same version
- **WHEN** user runs `skittle apply --skill openspec/explore` and it's already applied with identical content
- **THEN** the skill SHALL be silently skipped and counted as unchanged

### Requirement: Install records provenance
When a skill is installed to a target, the system SHALL record provenance in the registry: the source name, plugin name, skill name, and relative origin path. This provenance SHALL be used by `skittle collect` to map skills back to their source.

#### Scenario: Provenance recorded on install
- **WHEN** user runs `skittle install --skill legal/contract-review --target claude`
- **THEN** the registry SHALL record that `contract-review` on target `claude` originated from `external/anthropic-plugins/legal/skills/contract-review`

#### Scenario: Provenance recorded for managed plugin
- **WHEN** user runs `skittle install --skill my-tools/code-review --target claude`
- **AND** `code-review` is in `plugins/my-tools/skills/code-review`
- **THEN** the registry SHALL record the origin as `plugins/my-tools/skills/code-review`

#### Scenario: Provenance survives reinstall
- **WHEN** a skill is reinstalled (updated) on a target
- **THEN** the provenance SHALL be updated to reflect the current origin path

### Requirement: Uninstall
The CLI SHALL support `skittle uninstall` with `--skill <plugin/skill>`, `--plugin <name>`, or `--bundle <name>`. Optional `--target <name>`. Running `skittle uninstall` with no flags SHALL display help. All uninstall operations SHALL default to preview mode and require `--force` to execute.

#### Scenario: Uninstall specific skill
- **WHEN** user runs `skittle uninstall --skill openspec/explore --force`
- **THEN** "openspec/explore" SHALL be removed from all targets where it's installed

#### Scenario: Uninstall from specific target
- **WHEN** user runs `skittle uninstall --skill openspec/explore --target claude-global --force`
- **THEN** "openspec/explore" SHALL be removed only from "claude-global"

#### Scenario: Uninstall bundle
- **WHEN** user runs `skittle uninstall --bundle work-dev --force`
- **THEN** all skills from "work-dev" SHALL be uninstalled from all targets where they were installed by that bundle
- **THEN** the active bundle tracking for affected targets SHALL be cleared

#### Scenario: Uninstall without --force
- **WHEN** user runs `skittle uninstall --skill openspec/explore` without `--force`
- **THEN** the CLI SHALL display what would be removed without making changes
