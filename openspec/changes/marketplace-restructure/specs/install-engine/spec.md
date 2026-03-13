## MODIFIED Requirements

### Requirement: Install records provenance
When a skill is installed to a target, the system SHALL record provenance in the registry: the source name, plugin name, skill name, and relative origin path. This provenance SHALL be used by `skittle collect` to map skills back to their source.

#### Scenario: Provenance recorded on install
- **WHEN** user runs `skittle install --skill legal/contract-review --target claude`
- **THEN** the registry SHALL record that `contract-review` on target `claude` originated from `external/anthropic-plugins/legal/skills/contract-review`

#### Scenario: Provenance recorded for managed plugin
- **WHEN** user runs `skittle install --skill my-tools/code-review --target claude`
- **AND** `code-review` is in `plugins/my-tools/skills/code-review`
- **THEN** the registry SHALL record the origin as `plugins/my-tools/skills/code-review`

#### Scenario: Provenance survives reinstall
- **WHEN** a skill is reinstalled (updated) on a target
- **THEN** the provenance SHALL be updated to reflect the current origin path
