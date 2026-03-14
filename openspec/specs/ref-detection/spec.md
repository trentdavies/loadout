### Requirement: Detect ref type
The system SHALL determine whether a git ref is a tag or a branch by running `git tag --list <ref>` in the cloned repository. If the command produces output, the ref is a tag. Otherwise, it is a branch.

#### Scenario: Ref is a tag
- **WHEN** a source has `ref = "v2.0"` and the cloned repo contains a tag named `v2.0`
- **THEN** the system SHALL classify the ref as a tag (pinned)

#### Scenario: Ref is a branch
- **WHEN** a source has `ref = "develop"` and the cloned repo has no tag named `develop`
- **THEN** the system SHALL classify the ref as a branch (tracking)

#### Scenario: No ref specified
- **WHEN** a source has no ref configured
- **THEN** the system SHALL treat it as tracking the default branch (latest)
