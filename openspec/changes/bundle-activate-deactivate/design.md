## Context

Bundles currently track "active" state per target in `Registry.active_bundles`. This creates complexity: swap semantics, exclusive activation, state management. The new model treats bundles as stateless named groups — activate = batch install, deactivate = batch uninstall, both idempotent.

## Goals / Non-Goals

**Goals:**
- Replace swap with activate/deactivate
- Remove all active-bundle state tracking
- Make both operations idempotent (skip silently if already in desired state)
- Support `--all` flag for broadcast to all targets
- Maintain `--force` / dry-run pattern

**Non-Goals:**
- Tracking which bundles are "active" on which targets
- Preventing multiple bundles from being activated on the same target
- Rollback or undo semantics

## Decisions

### 1. Idempotent by design

Activate skips already-installed skills. Deactivate skips already-absent skills. No warnings, no errors — just a count of what was actually installed/uninstalled. This makes bundles safe to use in scripts and automation without worrying about pre-existing state.

### 2. Target validation is required

Both commands validate that the target exists in config before proceeding. `--all` iterates all configured targets (not just auto-sync ones — the user explicitly asked for all).

### 3. `--force` gates execution, same as current pattern

Without `--force`: show what would happen (dry run).
With `--force`: execute.
Global `-n` / `--dry-run` overrides `--force` back to dry run.

### 4. Complete removal of active_bundles

The `active_bundles` BTreeMap is removed from `Registry`. This means:
- Existing registry.json files with `active_bundles` will deserialize cleanly (serde skips unknown fields with `#[serde(default)]`)
- Actually, we should use `#[serde(deny_unknown_fields)]` check — if the Registry uses it, we need to handle migration. If not (likely with `default`), it just works.

### 5. CLI structure

```
bundle activate <bundle> <target>     [--force]
bundle activate <bundle> --all        [--force]
bundle deactivate <bundle> <target>   [--force]
bundle deactivate <bundle> --all      [--force]
```

`<target>` is an optional positional. `--all` is a boolean flag. Exactly one of target or `--all` must be provided — error if neither or both.

## Risks / Trade-offs

- **Breaking change**: Users relying on `bundle swap` or `active_bundles` state will need to migrate. Mitigation: swap was gated behind `--force` and likely low-usage.
- **No state means no "what's active" query**: You can't ask "which bundles are active on this target?" anymore. Mitigation: `skittle status` already shows installed skills per target. The bundle grouping was artificial.
