## MODIFIED Requirements

### Requirement: Init command
The CLI SHALL support `skittle init [url]` to initialize skittle configuration and data directories. When no URL is provided and stdin is a TTY, the CLI SHALL run an interactive wizard with the following steps after directory creation:

1. **Git init**: Prompt "Initialize git in skittle data dir? [Y/n]". If accepted, run `git init` in the data directory. Skip silently if `.git` already exists or `git` is not installed.
2. **Target detection**: Prompt "Detect and add agent targets? [Y/n]". If accepted, scan for agent installations and auto-add all found targets (no per-target prompting).
3. **Marketplace sources**: Present a multi-select list of known skill marketplaces. Fetch and register each selected marketplace as a source. Skip this step if a URL was provided as an argument.

In non-interactive mode (`--quiet` or non-TTY): git init (yes), detect targets (yes, auto-add), skip marketplaces.

#### Scenario: Init with interactive wizard
- **WHEN** user runs `skittle init` in a TTY with no URL
- **THEN** the CLI SHALL create directories and config
- **THEN** the CLI SHALL prompt for git init, target detection, and marketplace selection

#### Scenario: Init with URL skips marketplace prompt
- **WHEN** user runs `skittle init https://github.com/org/skills.git`
- **THEN** the CLI SHALL add the URL as a source
- **THEN** the CLI SHALL still prompt for git init and target detection
- **THEN** the CLI SHALL skip the marketplace selection prompt

#### Scenario: Init git already exists
- **WHEN** user runs `skittle init` and the data dir already has a `.git` directory
- **THEN** the CLI SHALL skip the git init step silently

#### Scenario: Init git not installed
- **WHEN** user runs `skittle init` and `git` is not available on PATH
- **THEN** the CLI SHALL skip the git init step silently (warn in verbose mode)

#### Scenario: Init target detection finds agents
- **WHEN** user accepts target detection and agent installations are found at `~/.claude`, `~/.codex`, etc.
- **THEN** the CLI SHALL add all found agent directories as targets automatically

#### Scenario: Init marketplace fetch failure
- **WHEN** user selects a marketplace and the fetch fails (network error, repo not found)
- **THEN** the CLI SHALL warn about the failure and continue with remaining selections (not abort)

#### Scenario: Init non-interactive
- **WHEN** user runs `skittle init --quiet`
- **THEN** the CLI SHALL create directories, git init, detect targets, and skip marketplaces without prompting

#### Scenario: Init already initialized
- **WHEN** user runs `skittle init` and config already exists
- **THEN** the CLI SHALL display a message indicating config already exists and exit
