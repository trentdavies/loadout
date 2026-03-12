## ADDED Requirements

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
- **WHEN** user runs `skittle add --help` or `skittle add -h` or `skittle add help`
- **THEN** the CLI SHALL display help for the `add` command including its arguments and flags

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

### Requirement: Add command
The CLI SHALL support `skittle add <url>` to register a new skill source. The URL MAY be a local path, a git URL, or a GitHub shorthand. An optional `--name <alias>` flag SHALL override the auto-derived source name.

#### Scenario: Add local directory source
- **WHEN** user runs `skittle add ~/my-skills --name my-skills`
- **THEN** the source SHALL be registered with name "my-skills" and the resolved absolute path
- **THEN** the source content SHALL be fetched and cached in the local registry

#### Scenario: Add git source
- **WHEN** user runs `skittle add https://github.com/org/agent-skills.git`
- **THEN** the source SHALL be cloned into the local cache
- **THEN** the source SHALL be registered with a name derived from the repo (e.g., "agent-skills")

#### Scenario: Add source with duplicate name
- **WHEN** user runs `skittle add <url>` and a source with the derived name already exists
- **THEN** the CLI SHALL exit with an error suggesting `--name` to use a different alias

### Requirement: Remove command
The CLI SHALL support `skittle remove <name>` to unregister a source and remove its cached content.

#### Scenario: Remove existing source
- **WHEN** user runs `skittle remove my-skills`
- **THEN** the source SHALL be removed from the config and its cached content deleted from the registry

#### Scenario: Remove source with installed skills
- **WHEN** user runs `skittle remove my-skills` and skills from that source are installed on targets
- **THEN** the CLI SHALL warn about installed skills and require `--force` to proceed

### Requirement: Update command
The CLI SHALL support `skittle update [name]` to fetch the latest content from sources. If no name is given, all sources SHALL be updated.

#### Scenario: Update specific source
- **WHEN** user runs `skittle update my-skills`
- **THEN** the CLI SHALL fetch the latest content from the source URL and update the local cache

#### Scenario: Update all sources
- **WHEN** user runs `skittle update`
- **THEN** the CLI SHALL fetch latest content from all registered sources

#### Scenario: Update with no changes
- **WHEN** user runs `skittle update my-skills` and the source is already up to date
- **THEN** the CLI SHALL display "my-skills: already up to date"

### Requirement: List command
The CLI SHALL support `skittle list` to display all registered sources with their URL, plugin count, and last-updated time. The CLI SHALL support `skittle list <name>` to display full details of a source including its plugins and skills.

#### Scenario: List all sources
- **WHEN** user runs `skittle list` and sources are registered
- **THEN** the CLI SHALL display a table with name, URL, plugin count, and last updated timestamp

#### Scenario: List with no sources
- **WHEN** user runs `skittle list` and no sources are registered
- **THEN** the CLI SHALL display a message indicating no sources and suggest `skittle add`

#### Scenario: List source details
- **WHEN** user runs `skittle list my-skills`
- **THEN** the CLI SHALL display the source URL, version, description, and a tree of plugins and their skills
