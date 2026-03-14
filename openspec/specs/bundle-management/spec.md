## ADDED Requirements

### Requirement: Create bundle
The CLI SHALL support `skittle bundle create <name>` to create a new empty bundle. Bundle names SHALL follow the same rules as skill names (lowercase, hyphens, no consecutive hyphens).

#### Scenario: Create new bundle
- **WHEN** user runs `skittle bundle create work-dev`
- **THEN** a bundle named "work-dev" SHALL be added to the config with an empty skills list

#### Scenario: Create duplicate bundle
- **WHEN** user runs `skittle bundle create work-dev` and a bundle with that name already exists
- **THEN** the CLI SHALL exit with an error: "Bundle 'work-dev' already exists"

### Requirement: Delete bundle
The CLI SHALL support `skittle bundle delete <name>` to remove a bundle definition.

#### Scenario: Delete existing bundle
- **WHEN** user runs `skittle bundle delete work-dev`
- **THEN** the bundle SHALL be removed from the config
- **THEN** skills installed by that bundle SHALL NOT be uninstalled from targets (they remain installed)

#### Scenario: Delete active bundle
- **WHEN** user runs `skittle bundle delete work-dev` and "work-dev" is the active bundle on one or more targets
- **THEN** the CLI SHALL warn about active usage and require `--force` to proceed

### Requirement: List bundles
The CLI SHALL support `skittle bundle list [PATTERN...]` to display bundles. When one or more patterns are provided, only bundles whose name matches any of the patterns SHALL be listed.

#### Scenario: List bundles
- **WHEN** user runs `skittle bundle list`
- **THEN** the CLI SHALL display each bundle's name, skill count, and active targets

#### Scenario: List bundles with filter
- **WHEN** user runs `skittle bundle list "dev-*"`
- **THEN** only bundles whose name matches `dev-*` SHALL be displayed

#### Scenario: List bundles with filter no matches
- **WHEN** user runs `skittle bundle list "nonexistent-*"`
- **THEN** no bundles SHALL be displayed and an informational message SHALL be shown

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

### Requirement: Swap bundles
The CLI SHALL support `skittle bundle swap <from> <to> [--target <name>]` to perform a clean replace: uninstall all skills from bundle `<from>`, then install all skills from bundle `<to>` on the specified targets. If no `--target` is given, the swap applies to all auto-sync targets.

#### Scenario: Swap bundles on all auto targets
- **WHEN** user runs `skittle bundle swap work-dev personal-writing`
- **THEN** all skills from "work-dev" SHALL be uninstalled from auto-sync targets
- **THEN** all skills from "personal-writing" SHALL be installed to auto-sync targets
- **THEN** the active bundle on those targets SHALL be updated to "personal-writing"

#### Scenario: Swap on specific target
- **WHEN** user runs `skittle bundle swap work-dev personal-writing --target claude-global`
- **THEN** the swap SHALL only apply to the "claude-global" target

#### Scenario: Swap with dry run
- **WHEN** user runs `skittle bundle swap work-dev personal-writing -n`
- **THEN** the CLI SHALL display what would be uninstalled and installed without making changes

### Requirement: Active bundle tracking
The registry SHALL track which bundle is active on each target. This is updated by `install --bundle` and `bundle swap`.

#### Scenario: Status shows active bundle
- **WHEN** user runs `skittle status` and targets have active bundles
- **THEN** the active bundle name SHALL be shown for each target
