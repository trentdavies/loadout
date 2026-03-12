## REMOVED Requirements

### Requirement: List skills
**Reason**: The `skill` subgroup is removed. Skill listing is now the top-level `skittle list` command. The `--source` and `--plugin` filter flags are dropped — users grep the table output instead.
**Migration**: Use `skittle list` instead of `skittle skill list`.

### Requirement: Show skill details
**Reason**: The `skill` subgroup is removed. Skill details are now accessed via `skittle list <plugin/skill>` which shows the same detail view (name, description, source, plugin, path).
**Migration**: Use `skittle list <plugin/skill>` instead of `skittle skill show <plugin/skill>`.
