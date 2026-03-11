## ADDED Requirements

### Requirement: Source operations have functional tests
The library API for source add, remove, list, show, and update SHALL be exercised through integration tests.

#### Scenario: Add local source and verify registry
- **WHEN** a local source is added via the library API
- **THEN** the registry contains the source with detected plugins and skills

#### Scenario: Add source with custom name
- **WHEN** a source is added with an explicit name override
- **THEN** the registry stores the source under the custom name

#### Scenario: Remove source cleans registry
- **WHEN** a source is removed
- **THEN** the registry no longer contains that source's plugins or skills

#### Scenario: Remove source with installed skills fails without force
- **WHEN** a source with installed skills is removed without `--force`
- **THEN** the operation returns an error

#### Scenario: List sources returns all registered sources
- **WHEN** multiple sources are added and then listed
- **THEN** all sources appear in the listing

#### Scenario: Show source displays detail
- **WHEN** `source show` is called for a registered source
- **THEN** it returns the source's URL, type, plugin count, and skill count

#### Scenario: Update source re-detects structure
- **WHEN** a source is updated after its content changes
- **THEN** the registry reflects the updated plugin and skill structure

### Requirement: Target operations have functional tests
The library API for target add, remove, list, show, and detect SHALL be exercised through integration tests.

#### Scenario: Add target with agent and path
- **WHEN** a target is added with agent type and path
- **THEN** the config contains the target entry

#### Scenario: Add target with scope and sync defaults
- **WHEN** a target is added without explicit scope or sync
- **THEN** the target defaults to scope "machine" and sync "auto"

#### Scenario: Remove target
- **WHEN** a target is removed
- **THEN** the config no longer contains that target

#### Scenario: Add duplicate target name fails
- **WHEN** a target is added with a name that already exists
- **THEN** the operation returns an error

#### Scenario: List targets returns all configured targets
- **WHEN** multiple targets are configured and listed
- **THEN** all targets appear in the listing

### Requirement: Plugin and skill queries have functional tests
The library API for plugin list/show and skill list/show SHALL be exercised through integration tests.

#### Scenario: List plugins across sources
- **WHEN** multiple sources are registered and plugins are listed
- **THEN** plugins from all sources appear

#### Scenario: List plugins filtered by source
- **WHEN** plugins are listed with a source filter
- **THEN** only plugins from that source appear

#### Scenario: Show plugin detail
- **WHEN** `plugin show` is called for a registered plugin
- **THEN** it returns the plugin's skills, source, and description

#### Scenario: List skills across sources
- **WHEN** multiple sources are registered and skills are listed
- **THEN** skills from all sources appear with their identity

#### Scenario: List skills filtered by plugin
- **WHEN** skills are listed with a plugin filter
- **THEN** only skills from that plugin appear

#### Scenario: Show skill detail
- **WHEN** `skill show` is called for a registered skill
- **THEN** it returns the skill's name, description, source, and plugin

#### Scenario: Show nonexistent skill returns error
- **WHEN** `skill show` is called for an identity not in the registry
- **THEN** the operation returns an error

### Requirement: Install operations have functional tests for all flag combinations
The install command SHALL be tested with `--skill`, `--plugin`, `--bundle`, and `--target` flags.

#### Scenario: Install specific skill by identity
- **WHEN** `install --skill plugin/skill-name` is executed
- **THEN** only that skill is installed to all targets

#### Scenario: Install specific plugin
- **WHEN** `install --plugin plugin-name` is executed
- **THEN** all skills in that plugin are installed to all targets

#### Scenario: Install bundle
- **WHEN** `install --bundle bundle-name` is executed with a configured bundle
- **THEN** all skills in the bundle are installed and the bundle is set as active

#### Scenario: Install to specific target
- **WHEN** `install --all --target target-name` is executed
- **THEN** skills are installed only to the specified target

#### Scenario: Install nonexistent skill fails
- **WHEN** `install --skill nonexistent/skill` is executed
- **THEN** the operation returns an error indicating the skill was not found

#### Scenario: Install nonexistent plugin fails
- **WHEN** `install --plugin nonexistent` is executed
- **THEN** the operation returns an error

#### Scenario: Uninstall specific skill
- **WHEN** `uninstall --skill plugin/skill-name` is executed after installation
- **THEN** only that skill is removed from all targets

#### Scenario: Uninstall bundle
- **WHEN** `uninstall --bundle bundle-name` is executed
- **THEN** all skills in the bundle are uninstalled and the active bundle is cleared

### Requirement: Bundle lifecycle has functional tests
Bundle create, delete, add, drop, swap, list, and show SHALL be tested through the library API.

#### Scenario: Create bundle and add skills
- **WHEN** a bundle is created and skills are added to it
- **THEN** the config contains the bundle with the specified skills

#### Scenario: Delete bundle
- **WHEN** a bundle is deleted
- **THEN** the config no longer contains the bundle

#### Scenario: Delete active bundle without force fails
- **WHEN** a bundle that is currently active is deleted without `--force`
- **THEN** the operation returns an error

#### Scenario: Drop skill from bundle
- **WHEN** a skill is dropped from a bundle
- **THEN** the bundle no longer contains that skill

#### Scenario: Swap bundle replaces installed skills
- **WHEN** `bundle swap` is executed from bundle A to bundle B
- **THEN** bundle A's skills are uninstalled and bundle B's skills are installed

#### Scenario: Create duplicate bundle name fails
- **WHEN** a bundle is created with a name that already exists
- **THEN** the operation returns an error

### Requirement: Status command has functional tests
The status command SHALL be tested through the library API.

#### Scenario: Status with sources, targets, and skills
- **WHEN** status is queried with registered sources and targets
- **THEN** it returns counts of sources, targets, plugins, skills, and installed skills

#### Scenario: Status with empty config
- **WHEN** status is queried with no sources or targets configured
- **THEN** it returns zero counts

### Requirement: Config and cache operations have functional tests
Config show and cache show/clean SHALL be tested through the library API.

#### Scenario: Config show returns current config
- **WHEN** config show is executed
- **THEN** it returns the current configuration contents

#### Scenario: Cache clean removes cached sources
- **WHEN** cache clean is executed after sources have been fetched
- **THEN** the cache directory is empty and cache size is 0
