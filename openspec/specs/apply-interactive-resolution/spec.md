## ADDED Requirements

### Requirement: Interactive prompt for changed skills
When `--interactive` or `-i` is specified and a skill has status CHANGED, the CLI SHALL prompt the user with options: `[s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit`.

#### Scenario: Interactive prompt displayed
- **WHEN** user runs `skittle apply --all -i` and skill "openspec/explore" has status CHANGED
- **THEN** the CLI SHALL display the skill name, its status, and the prompt `[s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit`

#### Scenario: Skip a skill
- **WHEN** user selects `s` at the interactive prompt
- **THEN** the skill SHALL not be modified at the agent and the CLI SHALL proceed to the next skill

#### Scenario: Overwrite a skill
- **WHEN** user selects `o` at the interactive prompt
- **THEN** the skill SHALL be overwritten at the agent and the CLI SHALL proceed to the next skill

#### Scenario: Force-all remaining
- **WHEN** user selects `f` at the interactive prompt
- **THEN** the current skill and all remaining CHANGED skills SHALL be overwritten without further prompting

#### Scenario: Quit
- **WHEN** user selects `q` at the interactive prompt
- **THEN** the CLI SHALL stop processing remaining skills, keep all changes made so far, and exit with code 0

### Requirement: Diff display
When user selects `d` at the interactive prompt, the CLI SHALL display a unified diff of all files in the skill directory, labeled `--- installed` and `+++ source`, grouped by filename.

#### Scenario: View diff
- **WHEN** user selects `d` at the interactive prompt for skill "openspec/explore"
- **THEN** the CLI SHALL display a unified diff for each differing file in the skill directory with headers showing the filename (e.g., `=== SKILL.md ===`)
- **THEN** the CLI SHALL re-display the prompt (without the diff option) as `[s]kip  [o]verwrite  [q]uit`

#### Scenario: Diff with multiple changed files
- **WHEN** the skill has changes in both SKILL.md and scripts/run.sh
- **THEN** the diff SHALL show both files separated by `=== <filename> ===` headers

### Requirement: New and unchanged skills skip prompting
In interactive mode, skills with status NEW SHALL be applied without prompting. Skills with status UNCHANGED SHALL be skipped without prompting.

#### Scenario: New skill in interactive mode
- **WHEN** user runs `skittle apply --all -i` and skill "openspec/verify" has status NEW
- **THEN** the skill SHALL be installed without prompting

#### Scenario: Unchanged skill in interactive mode
- **WHEN** user runs `skittle apply --all -i` and skill "openspec/verify" has status UNCHANGED
- **THEN** the skill SHALL be silently skipped without prompting

### Requirement: Apply summary
After all skills are processed, the CLI SHALL display a summary line in the format: "Applied N skills (X new, Y updated), skipped Z unchanged."

#### Scenario: Summary with mixed results
- **WHEN** an apply operation processes 2 new skills, 1 updated skill, and 3 unchanged skills
- **THEN** the CLI SHALL display: "Applied 3 skills (2 new, 1 updated), skipped 3 unchanged."

#### Scenario: Summary with skipped conflicts
- **WHEN** an apply operation in interactive mode skips 1 conflicting skill
- **THEN** the skipped conflict SHALL be counted separately: "Applied 2 skills (1 new, 1 updated), skipped 3 unchanged, 1 conflict skipped."
