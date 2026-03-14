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
- **WHEN** user runs `skittle bundle --help` or `skittle target --help`
- **THEN** the CLI SHALL display help for that command with all its subcommands listed

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
The CLI SHALL support `skittle add <url>` to register a new skill source. The URL MAY be a local path, a git URL, or a GitHub shorthand. Optional flags `--source <name>`, `--plugin <name>`, and `--skill <name>` SHALL override the auto-derived names at each level of the identity hierarchy. When override flags are not provided and stdin is a TTY, the CLI SHALL prompt the user to confirm or override each inferred name. When stdin is not a TTY or `--quiet` is passed, inferred defaults SHALL be used without prompting. For local directory sources, `--symlink` and `--copy` flags SHALL control whether the source is symlinked or copied into the cache. When neither flag is passed and the source is a local directory, the CLI SHALL prompt for the fetch mode with symlink as the default. In non-interactive mode, symlink SHALL be used. For SingleFile local sources, the CLI SHALL always copy without prompting and SHALL ignore `--symlink`/`--copy` flags.

#### Scenario: Add local directory source with symlink prompt
- **WHEN** user runs `skittle add ~/my-skills` in a TTY without `--symlink` or `--copy` and the source is a directory
- **THEN** the CLI SHALL prompt whether to symlink or copy, with symlink as the default
- **THEN** if the user accepts the default, the cache path SHALL be a symlink to the original source

#### Scenario: Add local directory source with --symlink flag
- **WHEN** user runs `skittle add ~/my-skills --symlink` and the source is a directory
- **THEN** the CLI SHALL create a symlink without prompting

#### Scenario: Add local directory source with --copy flag
- **WHEN** user runs `skittle add ~/my-skills --copy` and the source is a directory
- **THEN** the CLI SHALL copy the source into the cache without prompting

#### Scenario: Add local directory source non-interactive
- **WHEN** user runs `skittle add ~/my-skills` with stdin piped and the source is a directory
- **THEN** the CLI SHALL use symlink mode without prompting

#### Scenario: Add local single-file source always copies
- **WHEN** user runs `skittle add ~/skills/my-tool.md` (a single file)
- **THEN** the CLI SHALL copy the file into the cache without prompting for symlink/copy
- **THEN** the `--symlink` flag SHALL be silently ignored if passed

#### Scenario: Symlink/copy flags ignored for non-local sources
- **WHEN** user runs `skittle add https://github.com/org/repo.git --symlink`
- **THEN** the CLI SHALL ignore the `--symlink` flag and clone the git repo normally

#### Scenario: Add with interactive confirmation
- **WHEN** user runs `skittle add https://github.com/org/agent-skills.git` in a TTY
- **THEN** the CLI SHALL display the inferred source name and prompt the user to confirm or override it

#### Scenario: Add with all flags bypasses all prompts
- **WHEN** user runs `skittle add <url> --source s --plugin p --skill sk`
- **THEN** the CLI SHALL use the provided names without any interactive prompts

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
The CLI SHALL support `skittle update [name]` to fetch the latest content from sources. If no name is given, all sources SHALL be updated. For symlinked local sources, update SHALL skip re-fetch and only re-run detection and normalization.

#### Scenario: Update symlinked source
- **WHEN** user runs `skittle update my-skills` and the source has `mode: "symlink"`
- **THEN** the CLI SHALL skip fetching and re-run detect + normalize to pick up structural changes
- **THEN** the CLI SHALL print "(symlinked, re-detecting)" instead of the normal fetch message

#### Scenario: Update copied source
- **WHEN** user runs `skittle update my-skills` and the source has `mode: "copy"` or no mode set
- **THEN** the CLI SHALL re-fetch the source content as before

#### Scenario: Update all sources
- **WHEN** user runs `skittle update`
- **THEN** the CLI SHALL fetch latest content from all registered sources, respecting each source's mode

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
