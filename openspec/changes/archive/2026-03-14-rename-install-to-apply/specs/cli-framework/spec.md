## MODIFIED Requirements

### Requirement: Top-level command structure
The CLI SHALL provide these top-level commands: `init`, `add`, `remove`, `update`, `list`, `apply`, `uninstall`, `collect`, `status`, `bundle`, `target`, `config`.

#### Scenario: Running skittle with no arguments
- **WHEN** user runs `skittle` with no arguments
- **THEN** the CLI SHALL display the help text listing all top-level commands

#### Scenario: Running skittle with unknown command
- **WHEN** user runs `skittle foobar`
- **THEN** the CLI SHALL exit with a non-zero code and display an error with suggestions for similar commands
