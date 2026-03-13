## ADDED Requirements

### Requirement: Git source ref pinning
`skittle.toml` SHALL support an optional `ref` field on git sources. The `ref` field SHALL accept a tag name, branch name, or commit SHA. When present, `add` and `update` SHALL use the specified ref.

#### Scenario: Pin to a tag
- **WHEN** `skittle.toml` contains `ref = "v1.2.0"` for a git source
- **THEN** `skittle add` SHALL clone at that tag
- **THEN** `skittle update` SHALL fetch and checkout that tag

#### Scenario: Pin to a branch
- **WHEN** `skittle.toml` contains `ref = "main"` for a git source
- **THEN** `skittle update` SHALL fetch and checkout the latest commit on that branch

#### Scenario: Pin to a commit SHA
- **WHEN** `skittle.toml` contains `ref = "abc123def"` for a git source
- **THEN** `skittle update` SHALL fetch and checkout that exact commit

#### Scenario: No ref specified
- **WHEN** a git source has no `ref` field
- **THEN** the source SHALL track the default branch (HEAD)
