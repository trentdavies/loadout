## ADDED Requirements

### Requirement: CLI framework tests
Suite `00_cli_framework.sh` SHALL test: `skittle --help` exits 0 and lists all commands, `skittle -h` exits 0, `skittle help` exits 0, `skittle foobar` exits non-zero with error, `skittle install` with no flags exits non-zero and shows help.

#### Scenario: Help flags work
- **WHEN** `skittle --help` is run
- **THEN** exit code SHALL be 0 and stdout SHALL contain "install", "uninstall", "add", "remove", "update", "list", "agent", "bundle", "status", "config", "init"

#### Scenario: Unknown command errors
- **WHEN** `skittle foobar` is run
- **THEN** exit code SHALL be non-zero and stderr SHALL contain "error"

#### Scenario: Subcommand help
- **WHEN** `skittle bundle --help` is run
- **THEN** exit code SHALL be 0 and stdout SHALL contain "create", "delete", "list", "show", "add", "drop", "swap"

#### Scenario: Global flags accepted
- **WHEN** `skittle status --json` is run
- **THEN** exit code SHALL be 0 and stdout SHALL be valid JSON

#### Scenario: Dry run flag accepted
- **WHEN** `skittle install --all -n` is run
- **THEN** exit code SHALL be 0 and no files SHALL be written

### Requirement: Config management tests
Suite `01_config.sh` SHALL test: `skittle init` creates config file, `skittle init` on existing config shows message, `skittle config show` displays config, `skittle config show --json` outputs JSON.

#### Scenario: Init creates config
- **WHEN** `skittle init` is run in a clean environment
- **THEN** exit code SHALL be 0 and `$XDG_CONFIG_HOME/skittle/config.toml` SHALL exist

#### Scenario: Init idempotent
- **WHEN** `skittle init` is run twice
- **THEN** the second invocation SHALL exit with a message about existing config

#### Scenario: Config show works
- **WHEN** `skittle init` then `skittle config show` is run
- **THEN** exit code SHALL be 0 and stdout SHALL contain "skittle" or config content

### Requirement: Source management tests
Suite `02_source_management.sh` SHALL test: add local source, add git source (@network), remove source, update source, add duplicate name errors, remove with --force.

#### Scenario: Add local source
- **WHEN** `skittle add /fixtures/plugin-source --name test-plugin` is run
- **THEN** exit code SHALL be 0 and `skittle list` SHALL show skills from "test-plugin"

#### Scenario: Add git source
- **WHEN** `skittle add https://github.com/anthropics/courses.git --name anthropic` is run (requires @network)
- **THEN** exit code SHALL be 0 and `skittle list` SHALL show skills from "anthropic"

#### Scenario: Remove source
- **WHEN** `skittle remove test-plugin` is run
- **THEN** exit code SHALL be 0 and `skittle list` SHALL NOT show skills from "test-plugin"

#### Scenario: Duplicate name error
- **WHEN** `skittle add /fixtures/plugin-source` is run twice without `--name`
- **THEN** the second invocation SHALL exit non-zero with an error about duplicate name

### Requirement: Source detection tests
Suite `03_source_detection.sh` SHALL test all 5 detection paths: single SKILL.md file, flat directory with skill subdirs, plugin directory with plugin.json, full source with source.json, and unrecognizable directory (error case).

#### Scenario: Detect single file
- **WHEN** `skittle add /fixtures/single-skill/SKILL.md` is run
- **THEN** exit code SHALL be 0 and `skittle list` SHALL show one skill

#### Scenario: Detect flat directory
- **WHEN** `skittle add /fixtures/flat-skills/` is run
- **THEN** `skittle list` SHALL show 2 skills

#### Scenario: Detect plugin directory
- **WHEN** `skittle add /fixtures/plugin-source/` is run
- **THEN** `skittle list` SHALL show 3 skills

#### Scenario: Detect full source
- **WHEN** `skittle add /fixtures/full-source/` is run
- **THEN** `skittle list` SHALL show 3 skills

#### Scenario: Reject unrecognizable directory
- **WHEN** `skittle add /fixtures/invalid/empty-dir/` is run
- **THEN** exit code SHALL be non-zero and stderr SHALL contain an error about unrecognizable structure

### Requirement: Local registry tests
Suite `05_local_registry.sh` SHALL test: XDG paths are used correctly, registry.json is created and contains entries, cache directory structure mirrors sources, skill identity resolution (short form and disambiguation).

#### Scenario: Registry created on source add
- **WHEN** `skittle add` completes
- **THEN** `$XDG_DATA_HOME/skittle/registry.json` SHALL exist

#### Scenario: Cache mirrors source
- **WHEN** `skittle add /fixtures/plugin-source --name tp` is run
- **THEN** `$XDG_DATA_HOME/skittle/sources/tp/` SHALL exist and contain cached skill files

#### Scenario: Short-form skill identity
- **WHEN** `skittle list test-plugin/explore` is run
- **THEN** exit code SHALL be 0 and skill details SHALL be displayed

#### Scenario: Ambiguous skill identity
- **WHEN** two sources contain a plugin/skill with the same name
- **THEN** `skittle list <ambiguous>` SHALL exit non-zero and list the conflicting sources

### Requirement: Agent management tests
Suite `06_agent_management.sh` SHALL test: add agent with agent type and path, remove agent, list agents, show agent, agent scope and sync mode defaults.

#### Scenario: Add claude agent
- **WHEN** `skittle agent add claude /tmp/test-agents/claude --name test-claude --scope machine --sync auto` is run
- **THEN** exit code SHALL be 0 and `skittle agent list` SHALL show "test-claude"

#### Scenario: Add codex agent
- **WHEN** `skittle agent add codex /tmp/test-agents/codex --name test-codex` is run
- **THEN** exit code SHALL be 0

#### Scenario: Remove agent
- **WHEN** `skittle agent remove test-claude` is run
- **THEN** `skittle agent list` SHALL NOT show "test-claude"
- **THEN** `/tmp/test-agents/claude/` SHALL still exist (not deleted)

#### Scenario: Show agent details
- **WHEN** `skittle agent show test-claude` is run after adding and installing skills
- **THEN** stdout SHALL list installed skills

#### Scenario: Unknown agent type
- **WHEN** `skittle agent add unknown-agent /tmp/test-agents/x` is run with no custom adapter
- **THEN** exit code SHALL be non-zero

### Requirement: Agent adapter tests
Suite `07_agent_adapters.sh` SHALL test: claude adapter installs SKILL.md + supporting dirs, codex adapter works identically, custom TOML adapter respects config, unknown format errors.

#### Scenario: Claude adapter copies skill correctly
- **WHEN** a skill with `scripts/` is installed to claude agent
- **THEN** `/tmp/test-agents/claude/skills/<name>/SKILL.md` SHALL exist
- **THEN** `/tmp/test-agents/claude/skills/<name>/scripts/` SHALL exist

#### Scenario: Codex adapter copies skill correctly
- **WHEN** a skill is installed to codex agent
- **THEN** `/tmp/test-agents/codex/skills/<name>/SKILL.md` SHALL exist

#### Scenario: Custom adapter uses configured paths
- **WHEN** a custom adapter is defined with `skill_dir = "prompts/{name}"` and a skill is installed
- **THEN** the skill SHALL appear at `<agent>/prompts/<name>/SKILL.md`

### Requirement: Skill operations tests
Suite `08_skill_operations.sh` SHALL test: skill list, skill show via `list <name>`, Agent Skills spec validation (skip invalid frontmatter).

#### Scenario: Skill list shows all skills
- **WHEN** sources are added and `skittle list` is run
- **THEN** all skills from all sources SHALL be listed

#### Scenario: Skill show displays metadata
- **WHEN** `skittle list test-plugin/explore` is run
- **THEN** stdout SHALL contain name, description, and source information

#### Scenario: Invalid skills are skipped with warning
- **WHEN** a source containing `no-frontmatter/SKILL.md` is added
- **THEN** stderr SHALL contain a warning and the invalid skill SHALL NOT appear in `skittle list`

### Requirement: Install engine tests
Suite `09_install_engine.sh` SHALL test: install --all, install --skill, install --plugin, install --bundle, install --agent, uninstall --skill, uninstall --bundle, dry run (-n), idempotent install, install with no flags errors.

#### Scenario: Install requires flags
- **WHEN** `skittle install` is run with no flags
- **THEN** exit code SHALL be non-zero and stdout SHALL contain help text

#### Scenario: Install all to auto agents
- **WHEN** config has skills and auto-sync agents, and `skittle install --all` is run
- **THEN** skills SHALL appear in auto-sync agent directories

#### Scenario: Install specific skill
- **WHEN** `skittle install --skill test-plugin/explore --agent test-claude` is run
- **THEN** `/tmp/test-agents/claude/skills/explore/SKILL.md` SHALL exist

#### Scenario: Install plugin
- **WHEN** `skittle install --plugin test-plugin --agent test-claude` is run
- **THEN** all 3 skills from test-plugin SHALL be installed

#### Scenario: Uninstall skill
- **WHEN** `skittle uninstall --skill test-plugin/explore --agent test-claude` is run
- **THEN** `/tmp/test-agents/claude/skills/explore/` SHALL NOT exist

#### Scenario: Dry run writes nothing
- **WHEN** `skittle install --all -n` is run
- **THEN** exit code SHALL be 0 and no files SHALL be created in agent directories

#### Scenario: Idempotent install
- **WHEN** `skittle install --skill test-plugin/explore` is run twice
- **THEN** the second run SHALL succeed with an "already installed" or "up to date" message

### Requirement: Bundle management tests
Suite `10_bundle_management.sh` SHALL test: create, delete, list, show, add skills, drop skills, install bundle, swap bundles, active bundle tracking.

#### Scenario: Create bundle
- **WHEN** `skittle bundle create test-bundle` is run
- **THEN** exit code SHALL be 0 and `skittle bundle list` SHALL show "test-bundle"

#### Scenario: Add skills to bundle
- **WHEN** `skittle bundle add test-bundle test-plugin/explore test-plugin/apply` is run
- **THEN** `skittle bundle show test-bundle` SHALL list both skills

#### Scenario: Install bundle to agent
- **WHEN** `skittle install --bundle test-bundle --agent test-claude` is run
- **THEN** both skills SHALL be installed and `skittle status` SHALL show "test-bundle" as active on test-claude

#### Scenario: Swap bundles
- **WHEN** bundle-a has skills [explore, apply] and bundle-b has skills [verify], and `skittle bundle swap bundle-a bundle-b --agent test-claude` is run
- **THEN** explore and apply SHALL be uninstalled, verify SHALL be installed, and active bundle on test-claude SHALL be "bundle-b"

#### Scenario: Delete bundle
- **WHEN** `skittle bundle delete test-bundle` is run
- **THEN** `skittle bundle list` SHALL NOT show "test-bundle"

#### Scenario: Drop skill from bundle
- **WHEN** `skittle bundle drop test-bundle test-plugin/explore` is run
- **THEN** `skittle bundle show test-bundle` SHALL NOT list "test-plugin/explore"

### Requirement: End-to-end workflow test
Suite `11_end_to_end.sh` SHALL test the complete workflow: `skittle init` → `skittle add` (local fixture) → `skittle agent add` (mock claude + codex) → `skittle bundle create` + `skittle bundle add` → `skittle install --bundle` → `skittle status` (verify) → `skittle bundle swap` → `skittle uninstall --bundle` → `skittle remove`.

#### Scenario: Full lifecycle
- **WHEN** the complete workflow is executed in sequence
- **THEN** each step SHALL exit 0, intermediate filesystem state SHALL be validated, and final state SHALL be clean (no installed skills, no sources)

#### Scenario: Status reflects state at each step
- **WHEN** `skittle status --json` is run after each workflow step
- **THEN** the JSON SHALL reflect the current number of sources, agents, installed skills, and active bundles
