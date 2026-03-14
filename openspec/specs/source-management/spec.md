## ADDED Requirements

### Requirement: Add caches to external directory
The `skittle add` command SHALL cache external sources in the `external/` directory instead of `sources/`.

#### Scenario: Add git source
- **WHEN** user runs `skittle add git@github.com:org/repo.git`
- **THEN** the source SHALL be cloned into `external/<name>/`

#### Scenario: Add local source
- **WHEN** user runs `skittle add ~/my-skills`
- **THEN** the source SHALL be copied into `external/<name>/`

### Requirement: Ref support on add
The `skittle add` command SHALL support an optional `--ref` flag for git sources. When provided, the clone SHALL use `git clone --branch <ref> --depth 1` to fetch only that ref. The ref SHALL be stored in config. The post-clone `git checkout` workaround SHALL be removed.

#### Scenario: Add with ref
- **WHEN** user runs `skittle add git@github.com:org/repo.git --ref v1.2.0`
- **THEN** the source SHALL be cloned with `git clone --branch v1.2.0 --depth 1`
- **THEN** config SHALL record `ref = "v1.2.0"` for that source

#### Scenario: Add without ref
- **WHEN** user runs `skittle add git@github.com:org/repo.git`
- **THEN** the source SHALL be cloned with `git clone --depth 1` (default branch)
- **THEN** config SHALL have no ref for that source

#### Scenario: Add with invalid ref
- **WHEN** user runs `skittle add git@github.com:org/repo.git --ref nonexistent`
- **THEN** the clone SHALL fail with an error indicating the ref was not found

### Requirement: Update respects ref
The `skittle update` command SHALL respect the `ref` field when updating git sources. If ref is a branch, it SHALL fetch and reset to `origin/<branch>`. If ref is a tag, it SHALL warn that the source is pinned and skip the update. If no ref, it SHALL fetch and reset to `origin/HEAD`.

#### Scenario: Update tracking branch
- **WHEN** user runs `skittle update my-source` and the source has `ref = "develop"`
- **THEN** the update SHALL run `git fetch origin` and `git reset --hard origin/develop`

#### Scenario: Update pinned tag
- **WHEN** user runs `skittle update my-source` and the source has `ref = "v1.2.0"` which is a tag
- **THEN** the CLI SHALL display a warning: "source 'my-source' is pinned to v1.2.0, skipping"
- **THEN** no git operations SHALL be performed on that source

#### Scenario: Update unpinned source
- **WHEN** user runs `skittle update my-source` and the source has no ref
- **THEN** the update SHALL fetch and reset to `origin/HEAD`

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

### Requirement: Switch ref via update
The `skittle update` command SHALL accept `--ref <new-ref>` to switch a source to a different version. This SHALL fetch from origin, checkout the new ref, update the stored ref in config, and re-detect skills.

#### Scenario: Switch from tag to tag
- **WHEN** user runs `skittle update my-source --ref v3.0` and the source is currently at `v2.0`
- **THEN** the system SHALL fetch, checkout `v3.0`, update config to `ref = "v3.0"`, and re-detect skills

#### Scenario: Switch from tag to latest
- **WHEN** user runs `skittle update my-source --ref latest`
- **THEN** the system SHALL remove the ref from config, fetch, reset to `origin/HEAD`, and re-detect skills

#### Scenario: Switch from latest to branch
- **WHEN** user runs `skittle update my-source --ref develop`
- **THEN** the system SHALL fetch, checkout `origin/develop`, update config to `ref = "develop"`, and re-detect skills

### Requirement: Display ref in output
The `skittle list` and `skittle status` commands SHALL display the active ref for git sources. Sources with no ref SHALL display "latest". Sources with a tag ref SHALL display the tag name. Sources with a branch ref SHALL display the branch name.

#### Scenario: List shows ref
- **WHEN** user runs `skittle list` and source "my-source" has `ref = "v2.0"`
- **THEN** the output SHALL include the ref information for that source

#### Scenario: Status shows ref
- **WHEN** user runs `skittle status` and sources have various refs configured
- **THEN** the status output SHALL show ref information per source
