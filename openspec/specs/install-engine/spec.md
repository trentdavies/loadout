## ADDED Requirements

### Requirement: Install with explicit flags
The CLI SHALL support `skittle install` with required flags to specify scope. Running `skittle install` with no flags SHALL display the help text (not perform any action).

#### Scenario: Install with no flags
- **WHEN** user runs `skittle install`
- **THEN** the CLI SHALL display the install command help and exit with a non-zero code

### Requirement: Install all
The CLI SHALL support `skittle install --all` to install all configured skills to all auto-sync targets.

#### Scenario: Install all to auto targets
- **WHEN** user runs `skittle install --all`
- **THEN** all skills referenced in the config (from bundles, individual skill entries, or plugin entries) SHALL be installed to all targets with sync mode `auto`

#### Scenario: Install all with target override
- **WHEN** user runs `skittle install --all --target proj-claude`
- **THEN** all configured skills SHALL be installed to only the "proj-claude" target (regardless of its sync mode)

### Requirement: Install skill
The CLI SHALL support `skittle install --skill <plugin/skill>` to install a specific skill. Optional `--target <name>` to limit to one target (defaults to all auto-sync targets).

#### Scenario: Install specific skill
- **WHEN** user runs `skittle install --skill openspec/explore`
- **THEN** "openspec/explore" SHALL be installed to all auto-sync targets

#### Scenario: Install skill to specific target
- **WHEN** user runs `skittle install --skill openspec/explore --target claude-global`
- **THEN** "openspec/explore" SHALL be installed only to "claude-global"

#### Scenario: Install nonexistent skill
- **WHEN** user runs `skittle install --skill openspec/nonexistent`
- **THEN** the CLI SHALL exit with an error: "Skill 'openspec/nonexistent' not found in registry"

### Requirement: Install plugin
The CLI SHALL support `skittle install --plugin <name>` to install all skills from a plugin. Optional `--target <name>`.

#### Scenario: Install plugin
- **WHEN** user runs `skittle install --plugin openspec`
- **THEN** all skills from the "openspec" plugin SHALL be installed to all auto-sync targets

### Requirement: Install bundle
The CLI SHALL support `skittle install --bundle <name>` to install all skills from a bundle. Optional `--target <name>`.

#### Scenario: Install bundle
- **WHEN** user runs `skittle install --bundle work-dev`
- **THEN** all skills in the "work-dev" bundle SHALL be installed to all auto-sync targets
- **THEN** "work-dev" SHALL be recorded as the active bundle on those targets

### Requirement: Dry run
All install and uninstall operations SHALL support `-n` / `--dry-run` which displays what would change without making modifications.

#### Scenario: Dry run install
- **WHEN** user runs `skittle install --all -n`
- **THEN** the CLI SHALL display a list of skills that would be installed and to which targets, without writing any files

### Requirement: Idempotent install
Installing a skill that is already installed on a target SHALL be a no-op (no error, no file modification unless the cached version is newer).

#### Scenario: Reinstall same version
- **WHEN** user runs `skittle install --skill openspec/explore` and it's already installed at the same version
- **THEN** the CLI SHALL display "openspec/explore: already installed on claude-global (up to date)"

#### Scenario: Install newer version
- **WHEN** user runs `skittle install --skill openspec/explore` and the cached version is newer than the installed version
- **THEN** the skill SHALL be updated on the target

### Requirement: Uninstall
The CLI SHALL support `skittle uninstall` with `--skill <plugin/skill>`, `--plugin <name>`, or `--bundle <name>`. Optional `--target <name>`. Running `skittle uninstall` with no flags SHALL display help.

#### Scenario: Uninstall specific skill
- **WHEN** user runs `skittle uninstall --skill openspec/explore`
- **THEN** "openspec/explore" SHALL be removed from all targets where it's installed

#### Scenario: Uninstall from specific target
- **WHEN** user runs `skittle uninstall --skill openspec/explore --target claude-global`
- **THEN** "openspec/explore" SHALL be removed only from "claude-global"

#### Scenario: Uninstall bundle
- **WHEN** user runs `skittle uninstall --bundle work-dev`
- **THEN** all skills from "work-dev" SHALL be uninstalled from all targets where they were installed by that bundle
- **THEN** the active bundle tracking for affected targets SHALL be cleared
