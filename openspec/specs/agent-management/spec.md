## ADDED Requirements

### Requirement: Add an agent
The CLI SHALL support `skittle agent add <type> [path]` to register an agent. The `<type>` argument specifies the agent type (e.g., "claude", "codex", or a custom adapter name). The optional `[path]` specifies the agent directory (defaults to the agent's standard global path). Flags: `--scope` (machine|repo, default: inferred from path), `--sync` (auto|explicit, default: explicit for repo, auto for machine), `--name <alias>`.

#### Scenario: Add machine-global agent
- **WHEN** user runs `skittle agent add claude`
- **THEN** an agent SHALL be registered with the default path for claude (e.g., `~/.claude`), scope "machine", and sync "auto"

#### Scenario: Add repo-scoped agent
- **WHEN** user runs `skittle agent add claude ./project/.claude --scope repo`
- **THEN** an agent SHALL be registered with the given path, scope "repo", and sync "explicit"

#### Scenario: Add agent with custom name
- **WHEN** user runs `skittle agent add claude ~/dev/proj/.claude --name proj-claude`
- **THEN** the agent SHALL be registered with name "proj-claude"

#### Scenario: Add agent with unknown agent type
- **WHEN** user runs `skittle agent add unknown-agent` and no adapter exists for "unknown-agent"
- **THEN** the CLI SHALL exit with an error listing available agent types and suggesting custom adapter definition

### Requirement: Remove an agent
The CLI SHALL support `skittle agent remove <name>`.

#### Scenario: Remove agent
- **WHEN** user runs `skittle agent remove claude-global`
- **THEN** the agent SHALL be removed from the config
- **THEN** installed skills on that agent SHALL NOT be deleted (they remain on disk)

### Requirement: List agents
The CLI SHALL support `skittle agent list` to display all registered agents with their agent type, path, scope, sync mode, and count of installed skills.

#### Scenario: List agents
- **WHEN** user runs `skittle agent list`
- **THEN** the CLI SHALL display a table of all agents with name, type, path, scope, sync, and installed skill count

### Requirement: Show agent details
The CLI SHALL support `skittle agent show <name>` to display an agent's configuration and all skills currently installed on it.

#### Scenario: Show agent
- **WHEN** user runs `skittle agent show claude-global`
- **THEN** the CLI SHALL display the agent config and list all installed skills with their versions and source plugins

### Requirement: Auto-detect agents
The CLI SHALL support `skittle agent detect` to scan for known agent configurations on the machine and in the current directory.

#### Scenario: Detect agents on machine
- **WHEN** user runs `skittle agent detect`
- **THEN** the CLI SHALL scan standard paths for known agents (e.g., `~/.claude`, `~/.codex`, `~/.cursor`)
- **THEN** the CLI SHALL scan the current directory for agent config directories (e.g., `./.claude`, `./.codex`)
- **THEN** the CLI SHALL display found agents and prompt the user to add them as agents

#### Scenario: No agents detected
- **WHEN** user runs `skittle agent detect` and no agent configurations are found
- **THEN** the CLI SHALL display "No agent configurations found" and suggest `skittle agent add`

### Requirement: Agent sync mode
Each agent SHALL have a sync mode: `auto` or `explicit`. Agents with sync mode `auto` SHALL be included when `skittle install --all` is run. Agents with sync mode `explicit` SHALL only be updated when specifically named via `--agent`.

#### Scenario: Install all with auto agents
- **WHEN** user runs `skittle install --all`
- **THEN** only agents with sync mode `auto` SHALL be updated

#### Scenario: Install to explicit agent
- **WHEN** user runs `skittle install --all --agent proj-claude` and proj-claude has sync "explicit"
- **THEN** proj-claude SHALL be included in the install operation
