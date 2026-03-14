## ADDED Requirements

### Requirement: TTY detection
The prompt module SHALL detect whether stdin is a TTY via `is_interactive()`. When stdin is not a TTY, all prompt functions SHALL return the default value without reading from stdin.

#### Scenario: Interactive terminal
- **WHEN** stdin is connected to a TTY
- **THEN** `is_interactive()` SHALL return true

#### Scenario: Piped input
- **WHEN** stdin is piped or redirected (e.g., `echo "" | skittle add ...`)
- **THEN** `is_interactive()` SHALL return false

### Requirement: Confirm or override prompt
The prompt module SHALL provide a `confirm_or_override(label, default)` function that displays the default value and accepts Enter to confirm or typed input to override.

#### Scenario: User accepts default
- **WHEN** `confirm_or_override("Source", "my-skills")` is called and the user presses Enter
- **THEN** the function SHALL return `"my-skills"`

#### Scenario: User types override
- **WHEN** `confirm_or_override("Source", "my-skills")` is called and the user types `"custom-name"` then Enter
- **THEN** the function SHALL return `"custom-name"`

#### Scenario: Non-interactive fallback
- **WHEN** `confirm_or_override("Source", "my-skills")` is called and stdin is not a TTY
- **THEN** the function SHALL return `"my-skills"` without prompting

### Requirement: Select from list prompt
The prompt module SHALL provide a `select_from(label, options)` function that displays a numbered list and accepts a selection.

#### Scenario: User selects an option
- **WHEN** `select_from("Source", ["alpha", "beta"])` is called and the user enters `2`
- **THEN** the function SHALL return `"beta"`

#### Scenario: Non-interactive fallback for selection
- **WHEN** `select_from("Source", ["alpha", "beta"])` is called and stdin is not a TTY
- **THEN** the function SHALL return an error indicating interactive input is required

### Requirement: Quiet mode suppresses prompts
When the `--quiet` flag is active, all prompt functions SHALL use default values without displaying prompts or reading stdin, behaving identically to the non-interactive fallback.

#### Scenario: Quiet mode with confirm
- **WHEN** `--quiet` is passed and `confirm_or_override("Source", "my-skills")` is called
- **THEN** the function SHALL return `"my-skills"` without any output

#### Scenario: Quiet mode with select
- **WHEN** `--quiet` is passed and `select_from("Source", [...])` is called
- **THEN** the function SHALL return an error indicating interactive input is required
