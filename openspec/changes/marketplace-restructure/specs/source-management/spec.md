## MODIFIED Requirements

### Requirement: Add caches to external directory
The `skittle add` command SHALL cache external sources in the `external/` directory instead of `sources/`.

#### Scenario: Add git source
- **WHEN** user runs `skittle add git@github.com:org/repo.git`
- **THEN** the source SHALL be cloned into `external/<name>/`

#### Scenario: Add local source
- **WHEN** user runs `skittle add ~/my-skills`
- **THEN** the source SHALL be copied into `external/<name>/`

### Requirement: Ref support on add
The `skittle add` command SHALL support an optional `--ref` flag for git sources. The ref SHALL be stored in `skittle.toml` and used during clone.

#### Scenario: Add with ref
- **WHEN** user runs `skittle add git@github.com:org/repo.git --ref v1.2.0`
- **THEN** the source SHALL be cloned at that ref
- **THEN** `skittle.toml` SHALL record `ref = "v1.2.0"` for that source

### Requirement: Update respects ref
The `skittle update` command SHALL respect the `ref` field when updating git sources. If ref is a branch, it fetches latest on that branch. If ref is a tag or commit, it checks out that exact ref.

#### Scenario: Update pinned source
- **WHEN** user runs `skittle update my-source` and the source has `ref = "v1.2.0"` in skittle.toml
- **THEN** the update SHALL fetch and checkout `v1.2.0`

#### Scenario: Update unpinned source
- **WHEN** user runs `skittle update my-source` and the source has no ref
- **THEN** the update SHALL fetch and checkout the latest default branch
