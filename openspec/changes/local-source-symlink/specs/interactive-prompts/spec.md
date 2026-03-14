## ADDED Requirements

### Requirement: Fetch mode prompt for local sources
The prompt module SHALL provide a way to ask the user whether to symlink or copy a local source, with symlink as the default.

#### Scenario: User accepts symlink default
- **WHEN** the fetch mode prompt is shown and the user presses Enter
- **THEN** the function SHALL return `"symlink"`

#### Scenario: User selects copy
- **WHEN** the fetch mode prompt is shown and the user selects copy
- **THEN** the function SHALL return `"copy"`

#### Scenario: Non-interactive fallback
- **WHEN** the fetch mode prompt would be shown but stdin is not a TTY
- **THEN** the function SHALL return `"symlink"` without prompting

#### Scenario: Quiet mode fallback
- **WHEN** `--quiet` is passed and the fetch mode prompt would be shown
- **THEN** the function SHALL return `"symlink"` without prompting
