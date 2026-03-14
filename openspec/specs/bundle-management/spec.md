### Requirement: Create bundle
The CLI SHALL support `skittle bundle create <name>` to create a new empty bundle. Bundle names SHALL follow the same rules as skill names (lowercase, hyphens, no consecutive hyphens).

#### Scenario: Create new bundle
- **WHEN** user runs `skittle bundle create work-dev`
- **THEN** a bundle named "work-dev" SHALL be added to the config with an empty skills list

#### Scenario: Create duplicate bundle
- **WHEN** user runs `skittle bundle create work-dev` and a bundle with that name already exists
- **THEN** the CLI SHALL exit with an error: "Bundle 'work-dev' already exists"

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

### Requirement: Show bundle
The CLI SHALL support `skittle bundle show <name>` to display a bundle's skills.

#### Scenario: Show bundle
- **WHEN** user runs `skittle bundle show work-dev`
- **THEN** the CLI SHALL list all skills in the bundle with their plugin, source, and version

### Requirement: Add skills to bundle
The CLI SHALL support `skittle bundle add <bundle> <skill...>` to add one or more skills to a bundle. Skills are specified as `plugin/skill` identifiers. Glob patterns (e.g., `openspec/*`) SHALL be supported to add all matching skills. When a glob pattern is used, all matching skill identities SHALL be resolved to their fully qualified form (`source:plugin/skill`) and stored individually in the bundle config.

#### Scenario: Add single skill
- **WHEN** user runs `skittle bundle add work-dev openspec/explore`
- **THEN** "openspec/explore" SHALL be added to the "work-dev" bundle's skill list

#### Scenario: Add multiple skills
- **WHEN** user runs `skittle bundle add work-dev openspec/explore openspec/apply`
- **THEN** both skills SHALL be added to the bundle

#### Scenario: Add with glob
- **WHEN** user runs `skittle bundle add work-dev "openspec/*"`
- **THEN** all skills from the "openspec" plugin SHALL be resolved to fully qualified identities and added to the bundle
- **THEN** the CLI SHALL display the count of skills matched and added

#### Scenario: Add with multiple glob patterns
- **WHEN** user runs `skittle bundle add work-dev "legal/*" "sales/*"`
- **THEN** all skills matching either pattern SHALL be resolved and added to the bundle

#### Scenario: Add nonexistent skill
- **WHEN** user runs `skittle bundle add work-dev openspec/nonexistent` and that skill does not exist in the registry
- **THEN** the CLI SHALL exit with an error: "Skill 'openspec/nonexistent' not found in registry"

#### Scenario: Add duplicate skill
- **WHEN** user runs `skittle bundle add work-dev openspec/explore` and it's already in the bundle
- **THEN** the CLI SHALL display "openspec/explore is already in bundle 'work-dev'" (not an error, just informational)

#### Scenario: Add with glob no matches
- **WHEN** user runs `skittle bundle add work-dev "nonexistent/*"` and no skills match
- **THEN** the CLI SHALL exit with an error indicating no skills matched the pattern

### Requirement: Drop skills from bundle
The CLI SHALL support `skittle bundle drop <bundle> <skill...>` to remove one or more skills from a bundle.

#### Scenario: Drop skill
- **WHEN** user runs `skittle bundle drop work-dev openspec/explore`
- **THEN** "openspec/explore" SHALL be removed from the bundle's skill list

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

