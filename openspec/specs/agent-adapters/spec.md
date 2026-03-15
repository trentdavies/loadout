## ADDED Requirements

### Requirement: Adapter trait
Each agent adapter SHALL implement skill installation, skill uninstallation, and installed skill listing for a specific agent type. The adapter determines how a canonical Agent Skills format skill directory is mapped into the agent's expected structure.

#### Scenario: Adapter installs skill
- **WHEN** `install` is called for a skill on an agent
- **THEN** the adapter for that agent's type SHALL copy/transform the skill into the agent directory using the agent's expected layout

#### Scenario: Adapter uninstalls skill
- **WHEN** `uninstall` is called for a skill on an agent
- **THEN** the adapter SHALL remove the skill's files from the agent directory

#### Scenario: Adapter lists installed skills
- **WHEN** `agent show` is called for an agent
- **THEN** the adapter SHALL scan the agent directory and return the list of installed skill names

### Requirement: Built-in claude adapter
The `claude` adapter SHALL install skills as passthrough copies of the Agent Skills format. Skills SHALL be placed at `{agent_path}/skills/{skill_name}/SKILL.md`. Supporting directories (`scripts/`, `references/`, `assets/`) SHALL be copied alongside the `SKILL.md`.

#### Scenario: Install skill to claude agent
- **WHEN** a skill "explore" is installed to a claude agent at `~/.claude`
- **THEN** the skill directory SHALL be copied to `~/.claude/skills/explore/SKILL.md`
- **THEN** any `scripts/`, `references/`, or `assets/` directories SHALL be copied to `~/.claude/skills/explore/`

### Requirement: Built-in codex adapter
The `codex` adapter SHALL install skills identically to the claude adapter: `{agent_path}/skills/{skill_name}/SKILL.md` with supporting directories.

#### Scenario: Install skill to codex agent
- **WHEN** a skill "explore" is installed to a codex agent at `~/.codex`
- **THEN** the skill directory SHALL be copied to `~/.codex/skills/explore/SKILL.md`

### Requirement: TOML-defined custom adapters
Users SHALL be able to define custom agent adapters in the config TOML. A custom adapter definition SHALL include: `skill_dir` (template string with `{name}` placeholder), `skill_file` (filename, default `SKILL.md`), `format` (string identifying the format converter, default `agentskills`), `copy_dirs` (array of directory names to copy, default `["scripts", "references", "assets"]`).

#### Scenario: Custom adapter in config
- **WHEN** config contains an adapter definition for "my-agent" with `skill_dir = "prompts/{name}"` and `skill_file = "prompt.md"`
- **THEN** installing skill "explore" to a "my-agent" agent SHALL create `{agent_path}/prompts/explore/prompt.md`

#### Scenario: Custom adapter with no extra dirs
- **WHEN** a custom adapter has `copy_dirs = []`
- **THEN** only the skill file SHALL be copied, not supporting directories

### Requirement: Format passthrough
In phase 1, the only supported format SHALL be `agentskills` which copies the `SKILL.md` content as-is without transformation. If a custom adapter specifies an unknown format, the CLI SHALL exit with an error listing available formats.

#### Scenario: Unknown format
- **WHEN** a custom adapter specifies `format = "mdc"`
- **THEN** the CLI SHALL exit with an error: "Unknown format 'mdc'. Available formats: agentskills"
