## MODIFIED Requirements

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
