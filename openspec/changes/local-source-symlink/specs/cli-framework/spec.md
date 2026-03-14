## MODIFIED Requirements

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
