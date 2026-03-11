## ADDED Requirements

### Requirement: Add a target
The CLI SHALL support `skittle target add <agent> [path]` to register a target. The `<agent>` argument specifies the agent type (e.g., "claude", "codex", or a custom adapter name). The optional `[path]` specifies the target directory (defaults to the agent's standard global path). Flags: `--scope` (machine|repo, default: inferred from path), `--sync` (auto|explicit, default: explicit for repo, auto for machine), `--name <alias>`.

#### Scenario: Add machine-global target
- **WHEN** user runs `skittle target add claude`
- **THEN** a target SHALL be registered with the default path for claude (e.g., `~/.claude`), scope "machine", and sync "auto"

#### Scenario: Add repo-scoped target
- **WHEN** user runs `skittle target add claude ./project/.claude --scope repo`
- **THEN** a target SHALL be registered with the given path, scope "repo", and sync "explicit"

#### Scenario: Add target with custom name
- **WHEN** user runs `skittle target add claude ~/dev/proj/.claude --name proj-claude`
- **THEN** the target SHALL be registered with name "proj-claude"

#### Scenario: Add target with unknown agent type
- **WHEN** user runs `skittle target add unknown-agent` and no adapter exists for "unknown-agent"
- **THEN** the CLI SHALL exit with an error listing available agent types and suggesting custom adapter definition

### Requirement: Remove a target
The CLI SHALL support `skittle target remove <name>`.

#### Scenario: Remove target
- **WHEN** user runs `skittle target remove claude-global`
- **THEN** the target SHALL be removed from the config
- **THEN** installed skills on that target SHALL NOT be deleted (they remain on disk)

### Requirement: List targets
The CLI SHALL support `skittle target list` to display all registered targets with their agent type, path, scope, sync mode, and count of installed skills.

#### Scenario: List targets
- **WHEN** user runs `skittle target list`
- **THEN** the CLI SHALL display a table of all targets with name, agent, path, scope, sync, and installed skill count

### Requirement: Show target details
The CLI SHALL support `skittle target show <name>` to display a target's configuration and all skills currently installed on it.

#### Scenario: Show target
- **WHEN** user runs `skittle target show claude-global`
- **THEN** the CLI SHALL display the target config and list all installed skills with their versions and source plugins

### Requirement: Auto-detect targets
The CLI SHALL support `skittle target detect` to scan for known agent configurations on the machine and in the current directory.

#### Scenario: Detect agents on machine
- **WHEN** user runs `skittle target detect`
- **THEN** the CLI SHALL scan standard paths for known agents (e.g., `~/.claude`, `~/.codex`, `~/.cursor`)
- **THEN** the CLI SHALL scan the current directory for agent config directories (e.g., `./.claude`, `./.codex`)
- **THEN** the CLI SHALL display found agents and prompt the user to add them as targets

#### Scenario: No agents detected
- **WHEN** user runs `skittle target detect` and no agent configurations are found
- **THEN** the CLI SHALL display "No agent configurations found" and suggest `skittle target add`

### Requirement: Target sync mode
Each target SHALL have a sync mode: `auto` or `explicit`. Targets with sync mode `auto` SHALL be included when `skittle install --all` is run. Targets with sync mode `explicit` SHALL only be updated when specifically named via `--target`.

#### Scenario: Install all with auto targets
- **WHEN** user runs `skittle install --all`
- **THEN** only targets with sync mode `auto` SHALL be updated

#### Scenario: Install to explicit target
- **WHEN** user runs `skittle install --all --target proj-claude` and proj-claude has sync "explicit"
- **THEN** proj-claude SHALL be included in the install operation
