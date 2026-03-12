## REMOVED Requirements

### Requirement: Add a source
**Reason**: The `source` subgroup is removed. Source add is now the top-level `skittle add <url>` command with identical behavior.
**Migration**: Use `skittle add <url> [--name <alias>]` instead of `skittle source add <url>`.

### Requirement: Remove a source
**Reason**: The `source` subgroup is removed. Source remove is now the top-level `skittle remove <name>` command with identical behavior.
**Migration**: Use `skittle remove <name> [--force]` instead of `skittle source remove <name>`.

### Requirement: List sources
**Reason**: The `source` subgroup is removed. Users don't need to manage sources as a separate concept — they manage skills. Source information is visible via `skittle status` and `skittle config show`.
**Migration**: Use `skittle status` to see source count, or `skittle config show` to see source details.

### Requirement: Show source details
**Reason**: The `source` subgroup is removed. Source details (plugins, skills) are visible through `skittle list` which shows all skills with their plugin and source columns.
**Migration**: Use `skittle list` to see all skills grouped by plugin and source.

### Requirement: Update sources
**Reason**: The `source` subgroup is removed. Source update is now the top-level `skittle update [name]` command with identical behavior.
**Migration**: Use `skittle update [name]` instead of `skittle source update [name]`.
