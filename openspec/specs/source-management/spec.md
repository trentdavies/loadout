## ADDED Requirements

### Requirement: Add a source
The CLI SHALL support `skittle source add <url>` to register a new skill source. The URL MAY be a local path (`file://`, `~/`, `./`), a git URL (`git://`, `https://...*.git`), or a GitHub shorthand (`github.com/org/repo`). An optional `--name <alias>` flag SHALL override the auto-derived source name.

#### Scenario: Add local directory source
- **WHEN** user runs `skittle source add ~/my-skills --name my-skills`
- **THEN** the source SHALL be registered in the config with name "my-skills" and the resolved absolute path
- **THEN** the source content SHALL be fetched and cached in the local registry

#### Scenario: Add git source
- **WHEN** user runs `skittle source add https://github.com/org/agent-skills.git`
- **THEN** the source SHALL be cloned into the local cache
- **THEN** the source SHALL be registered in the config with a name derived from the repo (e.g., "agent-skills")

#### Scenario: Add source with duplicate name
- **WHEN** user runs `skittle source add <url>` and a source with the derived name already exists
- **THEN** the CLI SHALL exit with an error suggesting `--name` to use a different alias

### Requirement: Remove a source
The CLI SHALL support `skittle source remove <name>` to unregister a source and remove its cached content.

#### Scenario: Remove existing source
- **WHEN** user runs `skittle source remove my-skills`
- **THEN** the source SHALL be removed from the config
- **THEN** the cached content SHALL be deleted from the registry

#### Scenario: Remove source with installed skills
- **WHEN** user runs `skittle source remove my-skills` and skills from that source are installed on targets
- **THEN** the CLI SHALL warn about installed skills and require `--force` to proceed

### Requirement: List sources
The CLI SHALL support `skittle source list` to display all registered sources with their URL, plugin count, and last-updated time.

#### Scenario: List with sources registered
- **WHEN** user runs `skittle source list` and sources are registered
- **THEN** the CLI SHALL display a table with name, URL, plugin count, and last updated timestamp

#### Scenario: List with no sources
- **WHEN** user runs `skittle source list` and no sources are registered
- **THEN** the CLI SHALL display a message indicating no sources and suggest `skittle source add`

### Requirement: Show source details
The CLI SHALL support `skittle source show <name>` to display full details of a source including its plugins and skills.

#### Scenario: Show existing source
- **WHEN** user runs `skittle source show my-skills`
- **THEN** the CLI SHALL display the source URL, version, description, and a tree of plugins and their skills

### Requirement: Update sources
The CLI SHALL support `skittle source update [name]` to fetch the latest content from sources. If no name is given, all sources SHALL be updated.

#### Scenario: Update specific source
- **WHEN** user runs `skittle source update my-skills`
- **THEN** the CLI SHALL fetch the latest content from the source URL and update the local cache

#### Scenario: Update all sources
- **WHEN** user runs `skittle source update`
- **THEN** the CLI SHALL fetch latest content from all registered sources

#### Scenario: Update with no changes
- **WHEN** user runs `skittle source update my-skills` and the source is already up to date
- **THEN** the CLI SHALL display "my-skills: already up to date"
