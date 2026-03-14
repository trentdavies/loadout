## ADDED Requirements

### Requirement: Collect skill from target back to source
The CLI SHALL support `skittle collect --skill <name> --target <target>` to copy a skill from an install target back to its origin location in the skittle data directory. The origin SHALL be determined from provenance data in the registry.

#### Scenario: Collect tracked skill back to external source
- **WHEN** user runs `skittle collect --skill contract-review --target claude`
- **AND** the registry shows contract-review originated from `external/anthropic-plugins/legal/skills/contract-review`
- **THEN** the skill directory SHALL be copied from the target back to that origin path

#### Scenario: Collect tracked skill back to plugins
- **WHEN** user runs `skittle collect --skill code-review --target claude`
- **AND** the registry shows code-review originated from `plugins/my-tools/skills/code-review`
- **THEN** the skill directory SHALL be copied from the target back to that origin path

#### Scenario: Collect skill with unknown provenance
- **WHEN** user runs `skittle collect --skill unknown-skill --target claude`
- **AND** the registry has no provenance for that skill on that target
- **THEN** the CLI SHALL report the skill is untracked and suggest `--adopt`

### Requirement: Adopt skill into plugins
The CLI SHALL support `skittle collect --skill <name> --target <target> --adopt` to copy a skill from a target into `plugins/`, making it a managed skill. A `plugin.json` SHALL be created if the destination plugin does not already have one. The marketplace.json SHALL be regenerated.

#### Scenario: Adopt an external skill
- **WHEN** user runs `skittle collect --skill contract-review --target claude --adopt`
- **THEN** the skill SHALL be copied to `plugins/<plugin>/skills/contract-review/`
- **THEN** a `.claude-plugin/plugin.json` SHALL be created if not present
- **THEN** marketplace.json SHALL be regenerated to include the adopted plugin

#### Scenario: Adopt an untracked skill
- **WHEN** user runs `skittle collect --skill standup-helper --target claude --adopt`
- **AND** standup-helper has no provenance in the registry
- **THEN** the skill SHALL be copied to `plugins/local/skills/standup-helper/`
- **THEN** marketplace.json SHALL be regenerated

### Requirement: Scan target for untracked skills
The CLI SHALL support `skittle collect --target <target>` (without `--skill`) to scan the target for all installed skills and show which are tracked vs untracked. Untracked skills SHALL be offered for adoption.

#### Scenario: Scan with mixed tracked and untracked
- **WHEN** user runs `skittle collect --target claude`
- **AND** the target has 2 tracked skills and 1 untracked skill
- **THEN** the CLI SHALL list tracked skills with their origin
- **THEN** the CLI SHALL list untracked skills
- **THEN** the CLI SHALL prompt to adopt untracked skills

#### Scenario: Scan with --force adopts all untracked
- **WHEN** user runs `skittle collect --target claude --force`
- **THEN** all untracked skills SHALL be adopted into `plugins/local/skills/` without prompting
