## MODIFIED Requirements

### Requirement: Top-level command structure
The CLI SHALL provide these top-level commands: `init`, `install`, `uninstall`, `status`, `source`, `plugin`, `skill`, `bundle`, `target`, `config`, `cache`, `add`, `list`.

#### Scenario: Running skittle with no arguments
- **WHEN** user runs `skittle` with no arguments
- **THEN** the CLI SHALL display the help text listing all top-level commands

#### Scenario: Running skittle with unknown command
- **WHEN** user runs `skittle foobar`
- **THEN** the CLI SHALL exit with a non-zero code and display an error with suggestions for similar commands

### Requirement: Global flags
The CLI SHALL support these global flags on all commands: `-n` / `--dry-run`, `-v` / `--verbose`, `-q` / `--quiet`, `--json`, `--config <path>`.

#### Scenario: Dry run flag on additive command
- **WHEN** user passes `-n` or `--dry-run` to an additive command (install, source add, source update, target add)
- **THEN** the CLI SHALL display what would change without making any modifications

#### Scenario: Dry run flag on destructive command
- **WHEN** user passes `-n` or `--dry-run` to a destructive command (uninstall, source remove, bundle delete, bundle swap, target remove, cache clean)
- **THEN** the flag SHALL be accepted but has no additional effect since destructive commands already default to preview mode

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

## ADDED Requirements

### Requirement: Automatic color detection
The CLI SHALL automatically detect whether to use color output. Color SHALL be disabled when `NO_COLOR` environment variable is set, when `--json` is passed, or when stdout is not a TTY. No `--color` flag SHALL be exposed to users.

#### Scenario: Output to terminal
- **WHEN** stdout is a TTY and `NO_COLOR` is not set
- **THEN** the CLI SHALL emit ANSI color codes

#### Scenario: Output piped
- **WHEN** stdout is not a TTY
- **THEN** the CLI SHALL not emit ANSI color codes

#### Scenario: NO_COLOR environment variable
- **WHEN** `NO_COLOR` is set in the environment
- **THEN** the CLI SHALL not emit ANSI color codes regardless of TTY status

### Requirement: Destructive commands default to preview
Destructive commands (uninstall, source remove, bundle delete, bundle swap, target remove, cache clean) SHALL default to preview mode — showing what would happen without executing. These commands SHALL require `--force` to actually perform the operation.

#### Scenario: Destructive command without --force
- **WHEN** user runs a destructive command without `--force` (e.g., `skittle uninstall --skill foo`)
- **THEN** the CLI SHALL display what would be changed
- **THEN** the CLI SHALL print "Use --force to <action>"
- **THEN** no modifications SHALL be made

#### Scenario: Destructive command with --force
- **WHEN** user runs a destructive command with `--force` (e.g., `skittle uninstall --skill foo --force`)
- **THEN** the CLI SHALL execute the operation

#### Scenario: --dry-run and --force both passed
- **WHEN** user passes both `--dry-run` and `--force` to a destructive command
- **THEN** `--dry-run` SHALL take precedence and no modifications SHALL be made
