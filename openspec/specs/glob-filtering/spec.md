### Requirement: Glob pattern matching against skill identities
The registry SHALL support matching skill identities against glob patterns. Patterns are matched against the full `source:plugin/skill` identity string using standard glob syntax (`*`, `?`, `[...]`).

#### Scenario: Match all skills from a source
- **WHEN** user provides pattern `anthropic:*/*`
- **THEN** all skills from the "anthropic" source SHALL be returned

#### Scenario: Match all skills in a plugin
- **WHEN** user provides pattern `*:legal/*`
- **THEN** all skills in any plugin named "legal" across all sources SHALL be returned

#### Scenario: Match skills by name prefix
- **WHEN** user provides pattern `*:*/code-*`
- **THEN** all skills whose name starts with "code-" SHALL be returned

#### Scenario: No matches
- **WHEN** user provides a pattern that matches no skills
- **THEN** an empty result set SHALL be returned

### Requirement: Short-form pattern expansion
When a glob pattern does not contain `:`, the system SHALL auto-prefix it with `*:` before matching. This is consistent with how short-form identities (`plugin/skill`) already imply "any source."

#### Scenario: Short-form glob expansion
- **WHEN** user provides pattern `legal/*`
- **THEN** the system SHALL expand it to `*:legal/*` and match accordingly

#### Scenario: Fully qualified pattern unchanged
- **WHEN** user provides pattern `anthropic:legal/*`
- **THEN** the system SHALL use it as-is without modification

### Requirement: Glob detection
An input string SHALL be treated as a glob pattern if it contains any of the characters `*`, `?`, or `[`. Otherwise it SHALL be treated as an exact identity.

#### Scenario: Input with wildcard treated as glob
- **WHEN** user provides `legal/*` to a command that supports globs
- **THEN** it SHALL be processed as a glob pattern

#### Scenario: Input without glob characters treated as exact
- **WHEN** user provides `legal/contract-review` to a command that supports globs
- **THEN** it SHALL be processed as an exact skill identity lookup

### Requirement: Filter skill list by glob patterns
`skittle list` SHALL accept zero or more pattern arguments. Each argument is either a glob pattern or an exact identity. When multiple patterns are provided, the results SHALL be the union of all matches (deduplicated). When no patterns are provided, all skills SHALL be listed.

#### Scenario: List with single glob pattern
- **WHEN** user runs `skittle list "*:legal/*"`
- **THEN** only skills matching the pattern SHALL be displayed

#### Scenario: List with multiple patterns
- **WHEN** user runs `skittle list "legal/*" "sales/*"`
- **THEN** skills matching either pattern SHALL be displayed, with no duplicates

#### Scenario: List with glob pattern and --json
- **WHEN** user runs `skittle list "*:legal/*" --json`
- **THEN** the JSON output SHALL contain only matching skills

#### Scenario: List with glob pattern no matches
- **WHEN** user runs `skittle list "nonexistent/*"`
- **THEN** no skills SHALL be displayed and an informational message SHALL be shown

#### Scenario: List with mixed exact and glob arguments
- **WHEN** user runs `skittle list "legal/contract-review" "sales/*"`
- **THEN** the exact match and all glob matches SHALL be displayed together
