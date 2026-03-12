## REMOVED Requirements

### Requirement: Cache clean
**Reason**: The `cache` subgroup is removed. Per-source cache cleanup happens via `skittle remove <name>`. No bulk cache clean command is needed at this stage.
**Migration**: Use `skittle remove <name>` to remove individual sources and their cache. For bulk cleanup, delete `~/.local/share/skittle/sources/` directly.

### Requirement: Cache show
**Reason**: The `cache` subgroup is removed. Cache information is not a primary user need.
**Migration**: Check `~/.local/share/skittle/sources/` directly for cache size information.
