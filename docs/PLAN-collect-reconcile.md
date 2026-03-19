# Collect And Reconcile Implementation Plan

This plan captures the agreed direction for making skill round-tripping across
agents feel deliberate and predictable.

## Product Decisions

- `equip collect` becomes the primary command.
- `equip agent collect` remains as a compatibility alias initially.
- `collect` supports `@agent`, `--agent`, `+kit`, and `--kit`.
- `collect` is selection-oriented; it does not rename or move canonical skills.
- Untracked skills are never implicitly written into an external source.
- `--adopt-local` means "copy this untracked skill into a local plugin and make
  it owned locally."
- `--link <identity>` means "treat this untracked installed copy as the
  canonical skill `<identity>` and collect into that canonical location."
- Dedicated move and rename commands are deferred.
- Filesystem reorgs happen manually in the equip repo, followed by
  `equip reconcile`.

## Phase 1: Normalize The Command Model

1. Add top-level `collect`.
2. Keep `agent collect` as an alias to the same handler.
3. Add `--kit` and `+kit` support to collect argument preprocessing and parsing.
4. Rename `--adopt` to `--adopt-local`.
5. Keep `--adopt` as a hidden compatibility alias for a transition period.
6. Update help text, completions, README, and command docs.

## Phase 2: Extract Collect Into A Planner

1. Move collect classification and execution logic out of the CLI handler.
2. Introduce a shared `CollectPlan` model.
3. Make the planner consume:
   - selected agent
   - resolved patterns
   - resolved kit skills
   - registry
   - source index
4. Keep the CLI responsible for rendering and prompt orchestration only.

## Phase 3: Add Kit-Aware Selection

1. Reuse the same resolution machinery used by equip for patterns and kits.
2. Support:
   - `equip collect @claude +developer`
   - `equip collect --agent claude --kit developer`
   - `equip collect @claude +developer "legal/*"`
3. Report kit-selected skills missing on the chosen agent as missing, not as
   errors.

## Phase 4: Make Ownership Explicit For Untracked Skills

1. Replace implicit untracked adoption with explicit action states.
2. Add `--adopt-local`.
3. Add `--to <plugin>` for local adoption targets.
4. Add `--link <identity>`.
5. For untracked installed skills:
   - do nothing by default in non-interactive mode
   - offer explicit resolution in interactive mode
6. For `--link`:
   - validate that the identity exists
   - copy from the selected agent skill dir into that canonical skill path
   - record provenance for that agent install
7. Matching external source candidates should only be suggested, never applied
   implicitly.

## Phase 5: Track Staleness Across Agents

1. Extend registry state to capture enough information to detect stale installed
   copies.
2. Add content fingerprints at install or collect time.
3. After collect, compute which other agents now have stale copies of the same
   canonical identity.
4. Report stale agents without auto-syncing in this phase.

## Phase 6: Add Reconcile

1. Add `equip reconcile`.
2. Rediscover configured sources from disk.
3. Refresh plugin and skill paths.
4. Regenerate local marketplace metadata.
5. Update registry source, plugin, and skill paths when identities still match.
6. Detect missing and newly discovered skills and present a plan.
7. Support `--source <name>`.
8. Apply changes by default and honor `--dry-run`.

## Phase 7: Tighten External-Source Round Trips

1. Distinguish local, copied external, cloned external, and symlinked external
   targets in collect output.
2. Permit tracked skills to collect back to any writable canonical path.
3. Require explicit `--link` or `--adopt-local` for untracked skills.
4. Improve messaging for repo-backed external sources.

## Test Matrix

### Rust Tests

- collect parsing and target resolution with patterns and kits
- planner classification
- `--link` execution
- `--adopt-local --to <plugin>` execution
- stale-copy computation
- reconcile path rebasing

### Harness Tests

- top-level `collect`
- `agent collect` alias compatibility
- `@agent` and `+kit` expansion for collect
- explicit untracked handling
- manual filesystem move followed by `reconcile`

### Sandbox Tests

- collect tracked edits back into cloned external source
- collect untracked into a symlinked repo-backed source via `--link`
- collect from one agent and report another agent stale
- manual repo reorg followed by `reconcile`
