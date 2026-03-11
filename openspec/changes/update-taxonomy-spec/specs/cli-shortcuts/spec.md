## ADDED Requirements

### Requirement: skittle add shorthand
The CLI SHALL support `skittle add <source>` as a top-level shorthand that delegates to the same code path as `skittle source add <source>`. All flags supported by `skittle source add` SHALL be available on `skittle add`.

#### Scenario: Add source via shorthand
- **WHEN** user runs `skittle add ~/my-skills --name my-skills`
- **THEN** the behavior SHALL be identical to `skittle source add ~/my-skills --name my-skills`

#### Scenario: Add archive via shorthand
- **WHEN** user runs `skittle add ./plugin.skill`
- **THEN** the behavior SHALL be identical to `skittle source add ./plugin.skill`

### Requirement: skittle list shorthand
The CLI SHALL support `skittle list` as a top-level shorthand that delegates to `skittle skill list`. An optional `[skills]` argument SHALL be accepted but is the default behavior.

#### Scenario: List skills via shorthand
- **WHEN** user runs `skittle list`
- **THEN** the behavior SHALL be identical to `skittle skill list`

#### Scenario: List skills with explicit argument
- **WHEN** user runs `skittle list skills`
- **THEN** the behavior SHALL be identical to `skittle skill list`

### Requirement: skittle init with optional URL
The CLI SHALL support `skittle init [url]` where the optional URL argument specifies a GitHub repository or local path. When a URL is provided, `skittle init` SHALL initialize configuration AND populate the skittle cache (`~/.local/share/skittle/`) with the contents of the URL.

#### Scenario: Init without URL
- **WHEN** user runs `skittle init`
- **THEN** the CLI SHALL create the default config file and data directories (existing behavior)

#### Scenario: Init with GitHub URL
- **WHEN** user runs `skittle init https://github.com/org/agent-skills`
- **THEN** the CLI SHALL create the default config file and data directories
- **THEN** the CLI SHALL clone the repository contents into the skittle cache at `~/.local/share/skittle/`

#### Scenario: Init with local path
- **WHEN** user runs `skittle init ~/my-skills`
- **THEN** the CLI SHALL create the default config file and data directories
- **THEN** the CLI SHALL copy the path contents into the skittle cache at `~/.local/share/skittle/`

#### Scenario: Init with URL when already initialized
- **WHEN** user runs `skittle init https://github.com/org/skills` and config already exists
- **THEN** the CLI SHALL display "Config already exists" and suggest `skittle source add` instead
