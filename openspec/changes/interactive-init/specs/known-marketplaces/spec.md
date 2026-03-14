## ADDED Requirements

### Requirement: Curated marketplace list
The system SHALL maintain a list of known skill marketplaces as a const array of (display_name, git_url) tuples in `src/marketplace.rs`. The list SHALL be used by `init` to offer popular sources to the user.

#### Scenario: List contains valid entries
- **WHEN** the marketplace list is accessed
- **THEN** each entry SHALL have a non-empty display name and a valid git URL

#### Scenario: List is extensible
- **WHEN** a developer adds a new marketplace entry to the const array
- **THEN** it SHALL appear in the init wizard's multi-select prompt without other code changes
