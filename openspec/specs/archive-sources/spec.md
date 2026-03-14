## ADDED Requirements

### Requirement: SourceUrl supports archive files
`SourceUrl::parse` SHALL recognize `.zip` and `.skill` file extensions and produce a `SourceUrl::Archive(PathBuf)` variant. The path SHALL be resolved to an absolute path before storage.

#### Scenario: Parse .zip file path
- **WHEN** user provides `~/plugins/my-plugin.zip` as a source URL
- **THEN** `SourceUrl::parse` SHALL return `Archive` with the resolved absolute path

#### Scenario: Parse .skill file path
- **WHEN** user provides `./tools/helper.skill` as a source URL
- **THEN** `SourceUrl::parse` SHALL return `Archive` with the resolved absolute path

#### Scenario: Non-archive file falls through
- **WHEN** user provides `~/my-skill.md` as a source URL
- **THEN** `SourceUrl::parse` SHALL NOT return `Archive` (it SHALL return `Local` as before)

### Requirement: Fetch unpacks archives to cache
When `fetch()` receives a `SourceUrl::Archive`, it SHALL extract the archive contents into the cache directory. The cache directory SHALL contain the unpacked files, not the archive itself.

#### Scenario: Unpack zip to cache
- **WHEN** `fetch()` is called with a `.zip` archive source
- **THEN** the archive contents SHALL be extracted into the cache path
- **THEN** the cache path SHALL contain the unpacked directory tree

#### Scenario: Unpack .skill to cache
- **WHEN** `fetch()` is called with a `.skill` archive source
- **THEN** the archive SHALL be treated as a zip file and extracted into the cache path

#### Scenario: Archive file not found
- **WHEN** `fetch()` is called with an archive path that does not exist
- **THEN** the system SHALL return an error: "archive not found: <path>"

### Requirement: Archive size limits
The system SHALL enforce a maximum unpacked size of 100MB and a maximum file count of 10,000 files during archive extraction. Exceeding either limit SHALL abort extraction and return an error.

#### Scenario: Archive exceeds size limit
- **WHEN** an archive would unpack to more than 100MB
- **THEN** extraction SHALL abort with error: "archive exceeds maximum unpacked size (100MB)"

#### Scenario: Archive exceeds file count
- **WHEN** an archive contains more than 10,000 entries
- **THEN** extraction SHALL abort with error: "archive exceeds maximum file count (10,000)"

### Requirement: Archive contents classified by detection
After unpacking, the archive contents SHALL be passed to the standard detection pipeline. The detection stage SHALL have no knowledge that the contents came from an archive.

#### Scenario: Archive containing a plugin
- **WHEN** a `.skill` archive unpacks to a directory containing `plugin.toml` and skill subdirectories
- **THEN** detection SHALL classify it as `SinglePlugin`

#### Scenario: Archive containing an AgentSkill
- **WHEN** a `.zip` archive unpacks to a directory containing `SKILL.md` at the root
- **THEN** detection SHALL classify it as `SingleSkillDir`

#### Scenario: Archive containing loose AgentSkill contents
- **WHEN** a `.skill` archive unpacks to files including `SKILL.md` directly (no wrapping directory)
- **THEN** detection SHALL classify it as `SingleSkillDir`
