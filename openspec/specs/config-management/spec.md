## ADDED Requirements

### Requirement: Init command
The CLI SHALL support `skittle init` to create a default config file at the XDG config path (`~/.config/skittle/config.toml`) if one does not exist.

#### Scenario: First-time init
- **WHEN** user runs `skittle init` and no config exists
- **THEN** the CLI SHALL create `~/.config/skittle/config.toml` with commented-out example sections for sources, agents, adapters, and bundles

#### Scenario: Init with existing config
- **WHEN** user runs `skittle init` and a config already exists
- **THEN** the CLI SHALL exit with a message: "Config already exists at <path>. Use `skittle config edit` to modify."

### Requirement: Config show
The CLI SHALL support `skittle config show` to display the fully resolved configuration.

#### Scenario: Show config
- **WHEN** user runs `skittle config show`
- **THEN** the CLI SHALL display the resolved config with all sources, agents, adapters, and bundles

#### Scenario: Show config with JSON
- **WHEN** user runs `skittle config show --json`
- **THEN** the CLI SHALL output the resolved config as JSON

### Requirement: Config edit
The CLI SHALL support `skittle config edit` to open the config file in the user's `$EDITOR` (falling back to `$VISUAL`, then `vi`).

#### Scenario: Edit config
- **WHEN** user runs `skittle config edit`
- **THEN** the CLI SHALL open `~/.config/skittle/config.toml` in the user's editor

#### Scenario: No editor set
- **WHEN** user runs `skittle config edit` and neither `$EDITOR` nor `$VISUAL` is set
- **THEN** the CLI SHALL fall back to `vi`

### Requirement: Config file format
The config TOML SHALL support sections for: sources (`[[source]]`), agents (`[[agent]]`), custom adapters (`[adapter.<name>]`), and bundles (`[bundle.<name>]`).

#### Scenario: Complete config example
- **WHEN** a config file contains source, agent, adapter, and bundle sections
- **THEN** the CLI SHALL parse all sections and make them available to all commands

### Requirement: Status command
The CLI SHALL support `skittle status` to display a summary of the current state: registered sources, agents, installed skills, active bundles, and anything that's out of date.

#### Scenario: Full status
- **WHEN** user runs `skittle status`
- **THEN** the CLI SHALL display: source count and update status, agent count with active bundles, total installed skills, and any skills with newer versions available in the registry

#### Scenario: Status with JSON
- **WHEN** user runs `skittle status --json`
- **THEN** the CLI SHALL output the status as structured JSON
