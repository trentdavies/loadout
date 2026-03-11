## ADDED Requirements

### Requirement: Top-level command structure
The CLI SHALL provide these top-level commands: `init`, `install`, `uninstall`, `status`, `source`, `plugin`, `skill`, `bundle`, `target`, `config`, `cache`.

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
- **WHEN** user runs `skittle source --help` or `skittle source -h` or `skittle source help`
- **THEN** the CLI SHALL display help for the `source` command with all its subcommands listed

#### Scenario: Help on nested subcommand
- **WHEN** user runs `skittle source add --help`
- **THEN** the CLI SHALL display help for `source add` including its arguments and flags

### Requirement: Global flags
The CLI SHALL support these global flags on all commands: `-n` / `--dry-run`, `-v` / `--verbose`, `-q` / `--quiet`, `--json`, `--color <when>` (auto|always|never), `--config <path>`.

#### Scenario: Dry run flag
- **WHEN** user passes `-n` or `--dry-run` to any command that writes
- **THEN** the CLI SHALL display what would change without making any modifications

#### Scenario: JSON output
- **WHEN** user passes `--json` to any command
- **THEN** the CLI SHALL output machine-readable JSON instead of human-readable text

#### Scenario: Quiet mode
- **WHEN** user passes `-q` or `--quiet`
- **THEN** the CLI SHALL suppress all non-error output

#### Scenario: Verbose mode
- **WHEN** user passes `-v` or `--verbose`
- **THEN** the CLI SHALL output additional detail about operations being performed

#### Scenario: Custom config path
- **WHEN** user passes `--config /path/to/config.toml`
- **THEN** the CLI SHALL use that file instead of the default config location

#### Scenario: Color control
- **WHEN** user passes `--color never`
- **THEN** the CLI SHALL not emit ANSI color codes in output
- **WHEN** user passes `--color always`
- **THEN** the CLI SHALL emit ANSI color codes even when output is not a TTY

### Requirement: Exit codes
The CLI SHALL use exit code 0 for success and non-zero for errors.

#### Scenario: Successful command
- **WHEN** a command completes successfully
- **THEN** the CLI SHALL exit with code 0

#### Scenario: Failed command
- **WHEN** a command fails
- **THEN** the CLI SHALL exit with a non-zero code and print an error message to stderr
