## Context

Skittle identifies skills as `source:plugin/skill` strings. The registry holds all discovered skills and provides `find_skill(identity)` for exact lookup and `all_skills()` for full enumeration. Currently there's no way to match a subset by pattern.

Users with many sources will have hundreds of skills. They need filtering for discovery (`list`) and bulk selection (`bundle add`).

## Goals / Non-Goals

**Goals:**
- Glob pattern matching against the full `source:plugin/skill` identity string
- Short-form patterns (no `:`) auto-prefixed with `*:` for consistency with existing identity semantics
- Filter `skittle list` output by pattern
- Expand glob patterns in `skittle bundle add` skill arguments, storing fully resolved identities
- Filter `skittle bundle list` output by name pattern

**Non-Goals:**
- Segment-aware matching (no special treatment of `:` or `/` in glob semantics — they're just characters)
- Interactive selection from matched results
- Glob support in other commands (`apply`, `remove`, etc.) — can be added later

## Decisions

### Use `glob-match` crate for pattern matching
Match glob patterns against identity strings using `glob-match`. It's zero-dependency, provides a single `glob_match(pattern, input) -> bool` function, and supports `*`, `?`, and `[...]` character classes.

**Alternatives considered:**
- `globset` (from ripgrep): More powerful but pulls in regex — overkill for string matching
- Hand-rolled matcher: Simple but risks subtle bugs with edge cases like `**`, character classes
- Regex conversion: Unnecessary complexity

### Add `match_skills(pattern)` to Registry
A new method on `Registry` that iterates all skills, builds the full identity string, and tests it against the pattern. Returns `Vec<(&str, &RegisteredPlugin, &RegisteredSkill)>` — same shape as `all_skills()`.

**Short-form expansion** happens at the call site before passing to `match_skills()`: if the pattern contains no `:`, prepend `*:`.

### Detection: glob vs exact identity
If the input string contains `*`, `?`, or `[`, treat it as a glob pattern. Otherwise, use existing exact-match behavior. This is checked in the CLI handler, not in the registry method.

### `bundle list` filtering is name-only
`bundle list` filters on bundle names (simple strings), not skill identities. Uses the same `glob_match` function directly — no registry involvement.

## Risks / Trade-offs

- **Performance with hundreds of skills**: `match_skills` iterates all skills and builds identity strings each time. For hundreds of skills this is negligible. If it ever reaches thousands, we could cache identity strings — but not worth doing now.
- **Shell glob expansion**: Users must quote patterns containing `*` to prevent shell expansion. This is standard CLI behavior but worth noting in help text.
