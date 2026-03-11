## ADDED Requirements

### Requirement: Source detection has unit tests
The `source/detect.rs` module SHALL have unit tests covering `detect()`, `has_skill_frontmatter()`, `parse_skill_name()`, and `parse_skill_description()` for all four source structures and edge cases.

#### Scenario: Detect single-file source
- **WHEN** `detect()` is called on a path containing a single SKILL.md with valid frontmatter
- **THEN** it returns `SourceStructure::SingleFile`

#### Scenario: Detect flat-skills source
- **WHEN** `detect()` is called on a directory containing multiple SKILL.md files at the top level
- **THEN** it returns `SourceStructure::FlatSkills`

#### Scenario: Detect plugin source
- **WHEN** `detect()` is called on a directory containing a `plugin.toml`
- **THEN** it returns `SourceStructure::SinglePlugin`

#### Scenario: Detect full source
- **WHEN** `detect()` is called on a directory containing a `source.toml`
- **THEN** it returns `SourceStructure::FullSource`

#### Scenario: Detect with no skills
- **WHEN** `detect()` is called on an empty directory
- **THEN** it returns an error indicating no skills found

#### Scenario: Parse skill frontmatter with name only
- **WHEN** `has_skill_frontmatter()` is called on a SKILL.md with `name` but no `description`
- **THEN** it returns `true`

#### Scenario: Parse skill frontmatter missing entirely
- **WHEN** `has_skill_frontmatter()` is called on a file without YAML frontmatter delimiters
- **THEN** it returns `false`

#### Scenario: Parse skill name from frontmatter
- **WHEN** `parse_skill_name()` is called on a SKILL.md with `name: my-skill`
- **THEN** it returns `Some("my-skill")`

#### Scenario: Parse skill name when missing
- **WHEN** `parse_skill_name()` is called on a SKILL.md without a `name` field
- **THEN** it returns `None`

#### Scenario: Parse skill description when missing
- **WHEN** `parse_skill_description()` is called on a SKILL.md without a `description` field
- **THEN** it returns `None`

### Requirement: Source discovery has unit tests
The `source/discover.rs` module SHALL have unit tests covering `discover_plugins()` and `discover_skills()` for valid sources, empty directories, and edge cases.

#### Scenario: Discover plugins in multi-plugin source
- **WHEN** `discover_plugins()` is called on a source with multiple subdirectories containing skills
- **THEN** it returns a plugin entry for each valid subdirectory, sorted alphabetically

#### Scenario: Discover plugins skips hidden directories
- **WHEN** `discover_plugins()` is called on a source containing a `.hidden` directory
- **THEN** the hidden directory is not included in results

#### Scenario: Discover plugins in empty directory
- **WHEN** `discover_plugins()` is called on an empty directory
- **THEN** it returns an empty list

#### Scenario: Discover skills in plugin directory
- **WHEN** `discover_skills()` is called on a plugin directory with multiple SKILL.md files
- **THEN** it returns a skill entry for each valid SKILL.md, sorted alphabetically

#### Scenario: Discover skills with missing frontmatter
- **WHEN** `discover_skills()` is called on a directory containing a SKILL.md without valid frontmatter
- **THEN** the skill is skipped (not included in results)

#### Scenario: Discover skills in empty plugin
- **WHEN** `discover_skills()` is called on a directory with no SKILL.md files
- **THEN** it returns an empty list

### Requirement: Source fetch has unit tests for local paths
The `source/fetch.rs` module SHALL have unit tests covering `fetch()` for local file and directory sources.

#### Scenario: Fetch local directory source
- **WHEN** `fetch()` is called with a local directory path
- **THEN** it copies the directory contents to the cache and returns the cache path

#### Scenario: Fetch local single-file source
- **WHEN** `fetch()` is called with a local file path
- **THEN** it copies the file to the cache and returns the cache path

#### Scenario: Fetch nonexistent local path
- **WHEN** `fetch()` is called with a path that does not exist
- **THEN** it returns an error

#### Scenario: Recursive copy skips .git directory
- **WHEN** a source directory contains a `.git` subdirectory
- **THEN** the `.git` directory is not copied to the cache

### Requirement: Manifest parsing has unit tests
The `source/manifest.rs` module SHALL have unit tests covering `load_source_manifest()` and `load_plugin_manifest()` for valid, invalid, and edge-case TOML inputs.

#### Scenario: Load valid source manifest with [source] wrapper
- **WHEN** `load_source_manifest()` is called on a TOML file with `[source]` section containing `name`
- **THEN** it returns a `SourceManifest` with the correct name

#### Scenario: Load valid source manifest flat form
- **WHEN** `load_source_manifest()` is called on a TOML file with top-level `name` field
- **THEN** it returns a `SourceManifest` with the correct name

#### Scenario: Load source manifest missing name
- **WHEN** `load_source_manifest()` is called on a TOML file without a `name` field
- **THEN** it returns an error

#### Scenario: Load source manifest with empty name
- **WHEN** `load_source_manifest()` is called on a TOML file with `name = ""`
- **THEN** it returns an error

#### Scenario: Load source manifest file not found
- **WHEN** `load_source_manifest()` is called with a path that does not exist
- **THEN** it returns an error

#### Scenario: Load source manifest invalid TOML
- **WHEN** `load_source_manifest()` is called on a file with invalid TOML syntax
- **THEN** it returns an error

#### Scenario: Load valid plugin manifest
- **WHEN** `load_plugin_manifest()` is called on a valid `plugin.toml`
- **THEN** it returns a `PluginManifest` with the correct name

#### Scenario: Load plugin manifest missing name
- **WHEN** `load_plugin_manifest()` is called on a TOML file without a `name` field
- **THEN** it returns an error

### Requirement: Source normalize has unit tests
The `source/normalize.rs` module SHALL have unit tests covering `normalize()` for each `SourceStructure` variant.

#### Scenario: Normalize single-file source
- **WHEN** `normalize()` is called with `SourceStructure::SingleFile` and a valid SKILL.md
- **THEN** it returns a `RegisteredSource` with one plugin containing one skill

#### Scenario: Normalize flat-skills source
- **WHEN** `normalize()` is called with `SourceStructure::FlatSkills` and a directory of SKILL.md files
- **THEN** it returns a `RegisteredSource` with one plugin containing all discovered skills

#### Scenario: Normalize full source
- **WHEN** `normalize()` is called with `SourceStructure::FullSource` and a multi-plugin directory
- **THEN** it returns a `RegisteredSource` with multiple plugins each containing their skills

#### Scenario: Normalize single plugin source
- **WHEN** `normalize()` is called with `SourceStructure::SinglePlugin` and a plugin directory
- **THEN** it returns a `RegisteredSource` with one plugin

### Requirement: URL parsing has expanded unit tests
The `source/url.rs` module SHALL have additional unit tests covering relative paths, home expansion, and invalid URLs.

#### Scenario: Parse relative path with ./
- **WHEN** `SourceUrl::parse()` is called with `./local/path`
- **THEN** it returns a `Local` variant with the resolved absolute path

#### Scenario: Parse home-relative path
- **WHEN** `SourceUrl::parse()` is called with `~/some/path`
- **THEN** it returns a `Local` variant with the home directory expanded

#### Scenario: Parse invalid URL
- **WHEN** `SourceUrl::parse()` is called with an empty string or clearly malformed input
- **THEN** it returns an error

### Requirement: Config module has unit tests
The `config/mod.rs` module SHALL have unit tests for path resolution and load/save operations.

#### Scenario: Config path with override
- **WHEN** `config_path()` is called with `Some("/custom/path.toml")`
- **THEN** it returns the custom path

#### Scenario: Config path default
- **WHEN** `config_path()` is called with `None`
- **THEN** it returns a path under the XDG config directory

#### Scenario: Load config from nonexistent file returns default
- **WHEN** `load_from()` is called with a path that does not exist
- **THEN** it returns a default `Config`

#### Scenario: Load config from invalid TOML returns error
- **WHEN** `load_from()` is called with a file containing invalid TOML
- **THEN** it returns an error

#### Scenario: Save and reload config roundtrip
- **WHEN** a `Config` is saved with `save_to()` and reloaded with `load_from()`
- **THEN** the reloaded config matches the original

### Requirement: Config types have expanded unit tests
The `config/types.rs` module SHALL have additional tests for edge cases in deserialization.

#### Scenario: Deserialize adapter with format default
- **WHEN** a `[adapters.x]` section has no `format` field
- **THEN** the adapter's format defaults to `"agentskills"`

#### Scenario: Deserialize invalid TOML returns error
- **WHEN** `Config` is deserialized from syntactically invalid TOML
- **THEN** deserialization fails with an error

### Requirement: Registry module has expanded unit tests
The `registry/mod.rs` module SHALL have additional tests for `find_plugin()`, corrupted data, and persistence.

#### Scenario: Find plugin by name
- **WHEN** `find_plugin()` is called with a plugin name that exists in the registry
- **THEN** it returns the source name and plugin reference

#### Scenario: Find plugin not found
- **WHEN** `find_plugin()` is called with a name not in the registry
- **THEN** it returns `None`

#### Scenario: Load corrupted registry JSON
- **WHEN** `load_registry()` is called and the registry file contains invalid JSON
- **THEN** it returns an error

#### Scenario: Save and reload registry roundtrip
- **WHEN** a registry is saved with `save_registry()` and reloaded with `load_registry()`
- **THEN** the reloaded registry matches the original

### Requirement: Output module has unit tests
The `output/mod.rs` module SHALL have unit tests for flag gating and formatting logic.

#### Scenario: Quiet mode suppresses non-error output
- **WHEN** an `Output` is created with `quiet: true`
- **THEN** `success()`, `info()`, and `warn()` produce no output

#### Scenario: Verbose mode enables debug output
- **WHEN** an `Output` is created with `verbose: true`
- **THEN** `debug()` produces output

#### Scenario: Non-verbose mode suppresses debug
- **WHEN** an `Output` is created with `verbose: false`
- **THEN** `debug()` produces no output

#### Scenario: JSON mode emits valid JSON
- **WHEN** an `Output` is created with `json: true` and `json_value()` is called
- **THEN** the output is valid JSON

#### Scenario: Format size helper
- **WHEN** `format_size()` is called with various byte counts
- **THEN** it returns human-readable strings (e.g., "1.5 KB", "2.3 MB")

### Requirement: CLI helpers have unit tests
The `cli/mod.rs` helper functions `dir_size()` and `format_size()` SHALL have unit tests.

#### Scenario: dir_size on directory with files
- **WHEN** `dir_size()` is called on a directory containing files totaling 1024 bytes
- **THEN** it returns 1024

#### Scenario: dir_size on empty directory
- **WHEN** `dir_size()` is called on an empty directory
- **THEN** it returns 0

#### Scenario: dir_size on nonexistent path
- **WHEN** `dir_size()` is called on a path that does not exist
- **THEN** it returns 0
