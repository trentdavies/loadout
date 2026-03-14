## ADDED Requirements

### Requirement: Collect command
The CLI SHALL support `skittle collect` as a top-level command for copying skills from targets back to their origin.

#### Scenario: Collect specific skill
- **WHEN** user runs `skittle collect --skill <name> --target <target>`
- **THEN** the skill SHALL be copied from the target back to its origin path

#### Scenario: Collect with adopt
- **WHEN** user runs `skittle collect --skill <name> --target <target> --adopt`
- **THEN** the skill SHALL be copied into `plugins/` as a managed skill

#### Scenario: Collect all from target
- **WHEN** user runs `skittle collect --target <target>`
- **THEN** the CLI SHALL scan the target and show tracked vs untracked skills

#### Scenario: Collect help
- **WHEN** user runs `skittle collect --help`
- **THEN** the CLI SHALL display help for the collect command with all flags
