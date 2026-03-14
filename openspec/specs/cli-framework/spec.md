## ADDED Requirements

### Requirement: Top-level command structure
The CLI SHALL provide these top-level commands: `init`, `install`, `uninstall`, `status`, `source`, `plugin`, `skill`, `bundle`, `target`, `config`, `cache`, `add`, `list`.

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

### Requirement: Exit codes
The CLI SHALL use exit code 0 for success and non-zero for errors.

#### Scenario: Successful command
- **WHEN** a command completes successfully
- **THEN** the CLI SHALL exit with code 0

#### Scenario: Failed command
- **WHEN** a command fails
- **THEN** the CLI SHALL exit with a non-zero code and print an error message to stderr

### Requirement: Add command
The CLI SHALL support `skittle add <url>` to register a new skill source. The URL MAY be a local path, a git URL, or a GitHub shorthand. Optional flags `--source <name>`, `--plugin <name>`, and `--skill <name>` SHALL override the auto-derived names at each level of the identity hierarchy. When override flags are not provided and stdin is a TTY, the CLI SHALL prompt the user to confirm or override each inferred name. When stdin is not a TTY or `--quiet` is passed, inferred defaults SHALL be used without prompting.

#### Scenario: Add with interactive confirmation
- **WHEN** user runs `skittle add https://github.com/org/agent-skills.git` in a TTY
- **THEN** the CLI SHALL display the inferred source name (e.g., "agent-skills") and prompt the user to confirm or override it
- **THEN** after fetching and detecting structure, the CLI SHALL display inferred plugin and skill names and prompt for confirmation (skipping plugin prompt when plugin name equals source name)

#### Scenario: Add with --source flag bypasses source prompt
- **WHEN** user runs `skittle add https://github.com/org/agent-skills.git --source my-src`
- **THEN** the CLI SHALL use "my-src" as the source name without prompting for it
- **THEN** the CLI SHALL still prompt for plugin/skill names unless those flags are also provided

#### Scenario: Add with all flags bypasses all prompts
- **WHEN** user runs `skittle add <url> --source s --plugin p --skill sk`
- **THEN** the CLI SHALL use the provided names without any interactive prompts

#### Scenario: Add in non-interactive context
- **WHEN** user runs `skittle add <url>` with stdin piped or `--quiet` passed
- **THEN** the CLI SHALL use inferred defaults for all names without prompting
- **THEN** the CLI SHALL print the resolved identities to stderr (unless `--quiet`)

#### Scenario: Add local directory source
- **WHEN** user runs `skittle add ~/my-skills --source my-skills`
- **THEN** the source SHALL be registered with name "my-skills" and the resolved absolute path
- **THEN** the source content SHALL be fetched and cached in the local registry

#### Scenario: Add git source
- **WHEN** user runs `skittle add https://github.com/org/agent-skills.git`
- **THEN** the source SHALL be cloned into the local cache
- **THEN** the source SHALL be registered with a name derived from the repo (e.g., "agent-skills") after user confirmation

#### Scenario: Add source with duplicate name
- **WHEN** user runs `skittle add <url>` and a source with the derived name already exists
- **THEN** the CLI SHALL exit with an error suggesting `--source` to use a different alias

#### Scenario: Deprecated --name flag
- **WHEN** user runs `skittle add <url> --name foo`
- **THEN** the CLI SHALL exit with an error message: "`--name` has been renamed to `--source`"

### Requirement: Remove command
The CLI SHALL support `skittle remove [name]` to unregister a source and remove its cached content. When the name argument is omitted and stdin is a TTY, the CLI SHALL display registered sources and prompt the user to select one. When the name argument is omitted and stdin is not a TTY, the CLI SHALL exit with an error.

#### Scenario: Remove with positional name
- **WHEN** user runs `skittle remove my-skills`
- **THEN** the source SHALL be removed from the config and its cached content deleted from the registry

#### Scenario: Remove with interactive selection
- **WHEN** user runs `skittle remove` (no name) in a TTY with sources ["alpha", "beta", "gamma"] registered
- **THEN** the CLI SHALL display a numbered list of sources and prompt the user to select one
- **THEN** the selected source SHALL be removed (subject to `--force` if skills are installed)

#### Scenario: Remove without name in non-interactive context
- **WHEN** user runs `skittle remove` with stdin piped
- **THEN** the CLI SHALL exit with a non-zero code and an error message indicating a source name is required

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
