## REMOVED Requirements

### Requirement: List plugins
**Reason**: The `plugin` subgroup is removed from the CLI. Plugin is an internal organizational concept — users manage skills, not plugins. Plugin information is visible in the `list` table output (plugin column).
**Migration**: Use `skittle list` to see skills with their plugin grouping.

### Requirement: Show plugin details
**Reason**: The `plugin` subgroup is removed from the CLI. Plugin metadata is visible via `skittle list <plugin/skill>` which shows the plugin in the detail output.
**Migration**: Use `skittle list <plugin/skill>` to see plugin context for a skill.
