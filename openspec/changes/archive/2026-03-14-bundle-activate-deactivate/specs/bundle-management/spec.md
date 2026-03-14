## ADDED Requirements

### Requirement: Activate bundle
The CLI SHALL support `skittle bundle activate <bundle> <target>` to install all skills from the bundle onto the target. The CLI SHALL also support `--all` in place of a target to install onto all configured targets. Exactly one of `<target>` or `--all` MUST be provided. The operation SHALL be idempotent — skills already installed on the target SHALL be silently skipped. The command SHALL require `--force` to execute; without it, the CLI SHALL display what would be installed (dry run). The global `-n`/`--dry-run` flag SHALL override `--force`.

#### Scenario: Activate bundle on target
- **WHEN** user runs `skittle bundle activate work-dev my-claude --force`
- **THEN** all skills in "work-dev" SHALL be installed on the "my-claude" target
- **THEN** skills already installed on "my-claude" SHALL be silently skipped

#### Scenario: Activate bundle on all targets
- **WHEN** user runs `skittle bundle activate work-dev --all --force`
- **THEN** all skills in "work-dev" SHALL be installed on every configured target

#### Scenario: Activate dry run
- **WHEN** user runs `skittle bundle activate work-dev my-claude` (no --force)
- **THEN** the CLI SHALL display which skills would be installed without making changes

#### Scenario: Activate with invalid target
- **WHEN** user runs `skittle bundle activate work-dev nonexistent --force`
- **THEN** the CLI SHALL exit with an error: "target 'nonexistent' not found"

#### Scenario: Activate with invalid bundle
- **WHEN** user runs `skittle bundle activate nonexistent my-claude --force`
- **THEN** the CLI SHALL exit with an error: "bundle 'nonexistent' not found"

#### Scenario: Activate with neither target nor --all
- **WHEN** user runs `skittle bundle activate work-dev`
- **THEN** the CLI SHALL exit with an error indicating a target or --all is required

### Requirement: Deactivate bundle
The CLI SHALL support `skittle bundle deactivate <bundle> <target>` to uninstall all skills from the bundle from the target. The CLI SHALL also support `--all` in place of a target to uninstall from all configured targets. Exactly one of `<target>` or `--all` MUST be provided. The operation SHALL be idempotent — skills not installed on the target SHALL be silently skipped. The command SHALL require `--force` to execute; without it, the CLI SHALL display what would be uninstalled (dry run).

#### Scenario: Deactivate bundle from target
- **WHEN** user runs `skittle bundle deactivate work-dev my-claude --force`
- **THEN** all skills in "work-dev" SHALL be uninstalled from the "my-claude" target
- **THEN** skills not installed on "my-claude" SHALL be silently skipped

#### Scenario: Deactivate bundle from all targets
- **WHEN** user runs `skittle bundle deactivate work-dev --all --force`
- **THEN** all skills in "work-dev" SHALL be uninstalled from every configured target

#### Scenario: Deactivate dry run
- **WHEN** user runs `skittle bundle deactivate work-dev my-claude` (no --force)
- **THEN** the CLI SHALL display which skills would be uninstalled without making changes

## REMOVED Requirements

### Requirement: Swap bundles
**Reason**: Replaced by activate/deactivate. Swap implied exclusive activation; the new model allows multiple bundles to be active simultaneously.
**Migration**: Use `bundle deactivate <old> <target> --force && bundle activate <new> <target> --force`.

### Requirement: Active bundle tracking
**Reason**: Bundles are now stateless batch operations. There is no concept of an "active" bundle — activate is install, deactivate is uninstall.
**Migration**: Use `skittle status` to see which skills are installed on each target.

## MODIFIED Requirements

### Requirement: Delete bundle
The CLI SHALL support `skittle bundle delete <name>` to remove a bundle definition. The `--force` flag SHALL be required to confirm deletion. There is no active-bundle guard since active-bundle state no longer exists.

#### Scenario: Delete existing bundle
- **WHEN** user runs `skittle bundle delete work-dev --force`
- **THEN** the bundle SHALL be removed from the config
- **THEN** skills installed by that bundle SHALL NOT be uninstalled from targets (they remain installed)

#### Scenario: Delete without force
- **WHEN** user runs `skittle bundle delete work-dev` (no --force)
- **THEN** the CLI SHALL display what would be deleted without making changes

### Requirement: List bundles
The CLI SHALL support `skittle bundle list` to display all bundles with their skill count. There is no "ACTIVE ON" column.

#### Scenario: List bundles
- **WHEN** user runs `skittle bundle list`
- **THEN** the CLI SHALL display each bundle's name and skill count
