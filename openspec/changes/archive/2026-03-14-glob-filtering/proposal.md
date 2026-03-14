## Why

With potentially hundreds of skills across multiple sources, users need a way to filter and select skills by pattern rather than listing everything or specifying each one individually. The existing `skittle list` is all-or-one, `bundle add` requires exact identities, and `bundle list` has no filtering.

## What Changes

- `skittle list` accepts zero or more pattern arguments to filter skills by identity (e.g., `skittle list "legal/*" "sales/*"`)
- `skittle bundle add` expands glob patterns in skill arguments against the registry, storing fully resolved identities
- `skittle bundle list` accepts zero or more pattern arguments to filter bundles by name
- When a `list` or `bundle add` argument contains glob characters (`*`, `?`), it is treated as a pattern; otherwise existing exact-match behavior is preserved
- Short-form patterns without `:` are auto-prefixed with `*:` (e.g., `legal/*` becomes `*:legal/*`), consistent with how identity resolution already treats short-form identities
- A new dependency on `glob-match` (or similar lightweight glob crate) for pattern matching against identity strings

## Capabilities

### New Capabilities
- `glob-filtering`: Glob pattern matching against skill identity strings (`source:plugin/skill`) for filtering and selection

### Modified Capabilities
- `bundle-management`: `bundle add` expands globs and stores resolved identities; `bundle list` accepts optional name filter pattern

## Impact

- `src/registry/mod.rs`: New `match_skills(pattern)` method on `Registry`
- `src/cli/mod.rs`: Updated handlers for `List`, `BundleCommand::Add`, `BundleCommand::List`
- `Cargo.toml`: New dependency on glob matching crate
