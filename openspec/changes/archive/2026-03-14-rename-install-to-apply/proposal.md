## Why

The `install` command blindly overwrites target files with no detection or warning. If a user has hand-edited a skill at the target, those changes are silently lost. The command name "install" also implies a one-shot operation rather than an ongoing reconciliation. Renaming to `apply` and adding overwrite protection makes the default behavior safe while giving power users escape hatches.

## What Changes

- **BREAKING**: Rename `install` command to `apply`
- **BREAKING**: Default behavior now refuses to overwrite existing skills that differ from source — errors out with suggestion to use `--force` or `-i`
- Add `--force` / `-f` flag to overwrite all conflicts silently
- Add `--interactive` / `-i` flag for per-skill conflict resolution (skip/overwrite/diff/force-all/quit)
- Add byte-level skill directory comparison at apply time (no content hashing stored)
- New skills (not present at target) install without prompting in all modes
- Unchanged skills are silently skipped in all modes
- Summary line at end: "Applied N skills (X new, Y updated), skipped Z unchanged."
- `collect` command unchanged — always overwrites (git-managed source is not destructive)

## Capabilities

### New Capabilities
- `apply-conflict-detection`: Detect when source skills differ from installed skills at targets, with per-skill granularity (if any file in the skill directory differs, the whole skill is flagged as changed)
- `apply-interactive-resolution`: Interactive prompt for per-skill conflict resolution with skip/overwrite/diff/force-all/quit options, including unified diff display across all files in the skill directory

### Modified Capabilities
- `install-engine`: Rename from install to apply, add overwrite protection as default behavior, add --force and --interactive flags
- `cli-framework`: Replace `install` with `apply` in top-level command structure

## Impact

- **CLI**: `install` subcommand renamed to `apply` — breaking change for scripts/muscle memory
- **Code**: `src/cli/mod.rs` (command enum, run function), `src/target/adapter.rs` (comparison logic)
- **Specs**: `install-engine` and `cli-framework` specs need updates
- **Tests**: All install-related tests need renaming
