## MODIFIED Requirements

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
