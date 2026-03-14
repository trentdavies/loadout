## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Dry run applies to uninstall
**Reason**: Destructive commands now default to preview mode. `--dry-run` on uninstall is redundant since preview is the default behavior.
**Migration**: Uninstall without `--force` is equivalent to the old `--dry-run` behavior. Use `--force` to actually execute.
