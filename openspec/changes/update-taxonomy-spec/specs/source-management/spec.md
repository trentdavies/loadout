## MODIFIED Requirements

### Requirement: Add a source
The CLI SHALL support `skittle source add <url>` to register a new skill source. The URL MAY be a local path (`file://`, `~/`, `./`), a git URL (`git://`, `https://...*.git`), a GitHub shorthand (`github.com/org/repo`), a `.zip` file path, or a `.skill` file path. An optional `--name <alias>` flag SHALL override the auto-derived source name.

#### Scenario: Add local directory source
- **WHEN** user runs `skittle source add ~/my-skills --name my-skills`
- **THEN** the source SHALL be registered in the config with name "my-skills" and the resolved absolute path
- **THEN** the source content SHALL be fetched and cached in the local registry

#### Scenario: Add git source
- **WHEN** user runs `skittle source add https://github.com/org/agent-skills.git`
- **THEN** the source SHALL be cloned into the local cache
- **THEN** the source SHALL be registered in the config with a name derived from the repo (e.g., "agent-skills")

#### Scenario: Add zip file source
- **WHEN** user runs `skittle source add ~/downloads/my-plugin.zip`
- **THEN** the archive SHALL be unpacked into the local cache
- **THEN** the source SHALL be registered with a name derived from the filename ("my-plugin")

#### Scenario: Add .skill file source
- **WHEN** user runs `skittle source add ./tools/helper.skill`
- **THEN** the archive SHALL be unpacked into the local cache
- **THEN** the source SHALL be registered with a name derived from the filename ("helper")

#### Scenario: Add single SKILL.md file
- **WHEN** user runs `skittle source add ~/my-skill.md`
- **THEN** the file SHALL be copied into the local cache
- **THEN** the source SHALL be registered with a name derived from the filename ("my-skill")

#### Scenario: Add source with duplicate name
- **WHEN** user runs `skittle source add <url>` and a source with the derived name already exists
- **THEN** the CLI SHALL exit with an error suggesting `--name` to use a different alias
