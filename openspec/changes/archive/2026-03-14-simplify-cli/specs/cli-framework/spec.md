## MODIFIED Requirements

### Requirement: Top-level command structure
The CLI SHALL provide these top-level commands: `init`, `add`, `remove`, `update`, `list`, `install`, `uninstall`, `status`, `bundle`, `target`, `config`.

#### Scenario: Running skittle with no arguments
- **WHEN** user runs `skittle` with no arguments
- **THEN** the CLI SHALL display the help text listing all top-level commands

#### Scenario: Running skittle with unknown command
- **WHEN** user runs `skittle foobar`
- **THEN** the CLI SHALL exit with a non-zero code and display an error with suggestions for similar commands

### Requirement: Help available at every level
The CLI SHALL support `help`, `-h`, and `--help` at every command and subcommand level.

#### Scenario: Help on top-level
- **WHEN** user runs `skittle --help` or `skittle -h` or `skittle help`
- **THEN** the CLI SHALL display the top-level help text with all commands listed

#### Scenario: Help on subcommand
- **WHEN** user runs `skittle bundle --help` or `skittle target --help`
- **THEN** the CLI SHALL display help for that command with all its subcommands listed
