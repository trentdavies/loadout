## Why

Bundles are currently tied to a "swap" model with active-bundle state tracking per target. This is over-engineered — bundles should just be named groups of skills with batch install/uninstall semantics. Multiple bundles can be active simultaneously, and there's no state to track. Activate = batch install. Deactivate = batch uninstall. Both are idempotent.

## What Changes

- **Add `bundle activate <bundle> <target>` and `bundle activate <bundle> --all`**: Install all skills from the bundle onto the specified target (or all targets). Idempotent — silently skips skills already installed.
- **Add `bundle deactivate <bundle> <target>` and `bundle deactivate <bundle> --all`**: Uninstall all skills from the bundle from the specified target (or all targets). Idempotent — silently skips skills not installed.
- Both commands require `--force` to execute (dry run by default). Global `-n`/`--dry-run` overrides `--force`.
- **BREAKING**: Remove `bundle swap` command entirely.
- **BREAKING**: Remove `active_bundles` tracking from the registry. Drop `set_active_bundle`, `active_bundle`, `clear_active_bundle` methods.
- **Update `bundle list`**: Remove "ACTIVE ON" column. Show only bundle name and skill count.
- **Update `bundle delete`**: Remove the "active bundle" guard (no such concept). Always delete (no `--force` needed for active state, though `--force` may still be needed for other reasons).
- **Update `status`**: Remove active bundle display from status output.
- **Clean up references**: Remove all `active_bundles` reads/writes throughout CLI handlers.

## Capabilities

### New Capabilities
- (none — extends existing bundle-management)

### Modified Capabilities
- `bundle-management`: Replace swap with activate/deactivate. Remove active-bundle tracking. Make operations idempotent.

## Impact

- `src/cli/mod.rs`: Replace `BundleCommand::Swap` with `Activate` and `Deactivate`. Update `List`, `Delete`, and status handlers.
- `src/registry/types.rs`: Remove `active_bundles` field from `Registry`.
- `src/registry/mod.rs`: Remove `set_active_bundle`, `active_bundle`, `clear_active_bundle` methods and their tests.
- Shell test suites referencing swap or active bundles need updating.
