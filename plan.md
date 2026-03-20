# Plan: Native fzf Integration for equip

## Pros and Cons

### Pros
1. **Discoverability** — Users with large skill libraries can browse/search interactively instead of memorizing glob patterns
2. **Composability** — fzf output feeds naturally into other equip commands via pipes (`equip list --fzf | xargs equip @claude`)
3. **Multi-select** — Batch operations become trivial: select N skills visually, pipe into equip/remove/kit add
4. **Preview** — SKILL.md preview while browsing gives context without switching tools
5. **Low dependency cost** — fzf is an external binary (not compiled in), so no new Rust crate dependencies; users who don't have fzf simply get a clear error message
6. **Unix philosophy fit** — equip already shells out for git; fzf follows the same pattern
7. **Existing precedent** — There's already a `--fzf` flag on `equip list`, so the direction is established

### Cons
1. **External dependency** — fzf must be installed separately; not available by default on most systems
2. **Platform variance** — fzf behavior/availability differs across macOS, Linux, Windows (WSL)
3. **stdin contention** — When equip receives piped stdin, fzf cannot also read from stdin for user interaction (fzf reads from `/dev/tty` to work around this, but the equip side needs care)
4. **Testing difficulty** — Interactive fzf sessions are hard to test in CI; requires mock/skip strategies
5. **Version fragmentation** — Different fzf versions support different flags (e.g., `--multi` is universal, but `--bind` behaviors vary)
6. **Scope creep risk** — Once fzf is native, users will want fzf everywhere (agent select, source select, kit select, conflict resolution)

---

## Implementation Plan

### Phase 1: Core fzf Module (`src/fzf.rs`)

Create a reusable fzf integration module that all commands can share.

**File: `src/fzf.rs`**

```rust
pub struct FzfOptions {
    pub multi: bool,              // --multi for multi-select
    pub header: Option<String>,   // --header text
    pub preview: Option<String>,  // --preview command
    pub preview_window: Option<String>,
    pub prompt: Option<String>,   // --prompt text
    pub bind: Vec<String>,        // --bind key:action pairs
}
```

Key functions:
- `fzf_available() -> bool` — Check if fzf is in PATH
- `run_fzf(items: &[FzfItem], opts: &FzfOptions) -> Result<Vec<FzfItem>>` — Core runner
- Handles stdin/stdout plumbing correctly:
  - When equip's own stdin is a pipe (not TTY), read piped items from stdin, pass them to fzf
  - fzf reads user input from `/dev/tty` (it does this automatically)
  - Capture fzf's stdout for selected items

**Stdin behavior matrix:**

| equip stdin | fzf input source | User interaction |
|-------------|-----------------|-----------------|
| TTY (normal) | Items from equip registry | fzf reads /dev/tty |
| Pipe (`echo "dev*" \| equip list --fzf`) | Items from equip registry filtered by piped patterns | fzf reads /dev/tty |
| Pipe of identities (`equip list \| equip list --fzf`) | Piped lines become fzf input | fzf reads /dev/tty |

### Phase 2: Upgrade `equip list --fzf` (Existing Flag)

Enhance the existing `--fzf` implementation at `src/cli/commands/source.rs:706-760`:

1. **Multi-select support**: Add `--fzf-multi` or make `--fzf` support `-m` (multi-select by default)
2. **Better preview**: Show SKILL.md content with syntax highlighting header showing identity, source, description
3. **Action bindings**:
   - `enter` → print selected identity/identities to stdout
   - `ctrl-e` → equip selected to agent (invoke `equip @agent <identity>`)
   - `ctrl-r` → remove selected
   - `ctrl-y` → copy identity to clipboard
4. **Stdin integration**: When stdin is a pipe, accept skill identities as fzf input instead of listing all skills

### Phase 3: New `equip fzf` Top-Level Command

Add a dedicated `equip fzf` command that is the full interactive skill browser:

```
equip fzf [patterns...] [--agent <name>] [--action <equip|remove|print>]
```

- Default action: print selected identities to stdout
- With `--agent`: equip selected skills to that agent
- With `--action remove`: remove selected skills
- Reads from stdin if piped (filter/replace skill list)

**CLI definition** (in `src/cli/mod.rs`):

```rust
/// Interactive skill browser (requires fzf)
Fzf {
    /// Filter patterns
    patterns: Vec<String>,

    /// Agent to equip selected skills to
    #[arg(short, long)]
    agent: Option<String>,

    /// Action: print (default), equip, remove, kit-add
    #[arg(long, default_value = "print")]
    action: String,

    /// Multi-select (default: true)
    #[arg(long, default_value = "true")]
    multi: bool,
}
```

### Phase 4: fzf Integration Points Across Commands

Add `--fzf` flag to these existing commands:

1. **`equip collect --fzf`** — Replace dialoguer multi-select with fzf multi-select for skill collection
2. **`equip source remove --fzf`** — Browse sources with fzf, preview shows source details
3. **`equip kit add <name> --fzf`** — Select skills to add to a kit via fzf
4. **`equip @agent --fzf`** — Browse and select skills to equip interactively

### Phase 5: Stdin "Do the Right Thing" Logic

The stdin handling should follow this priority:

1. **No stdin, no patterns** → Show all skills from registry in fzf
2. **Patterns, no stdin** → Filter registry by patterns, show matches in fzf
3. **Stdin is pipe with identities** → Use piped identities as the fzf item list (enables `equip list "dev*" | equip fzf --agent claude`)
4. **Stdin is pipe + patterns** → Filter piped identities by patterns, then show in fzf
5. **Non-interactive (no TTY at all)** → Error with helpful message ("fzf requires a terminal")

Implementation in `src/fzf.rs`:
```rust
fn resolve_fzf_input(patterns: &[String], registry: &Registry) -> Result<Vec<FzfItem>> {
    let stdin_is_tty = std::io::stdin().is_terminal();

    if !stdin_is_tty {
        // Read identities from stdin pipe
        let mut items = read_stdin_identities()?;
        if !patterns.is_empty() {
            items = filter_by_patterns(items, patterns);
        }
        Ok(items)
    } else {
        // Get items from registry
        let skills = if patterns.is_empty() {
            registry.all_skills()
        } else {
            resolve_skill_patterns(patterns, registry, true)?
        };
        Ok(skills_to_fzf_items(skills))
    }
}
```

---

## File Changes Summary

| File | Change |
|------|--------|
| `src/fzf.rs` | **NEW** — Core fzf module |
| `src/main.rs` | Add `mod fzf;` |
| `src/cli/mod.rs` | Add `Fzf` command variant |
| `src/cli/commands/mod.rs` | Add `pub mod fzf;` |
| `src/cli/commands/fzf.rs` | **NEW** — `equip fzf` command implementation |
| `src/cli/commands/source.rs` | Refactor existing `--fzf` to use shared module, add multi-select |
| `src/cli/commands/collect.rs` | Add `--fzf` flag, use fzf for skill selection |
| `src/cli/commands/equip.rs` | Add `--fzf` flag for interactive skill browsing before equip |

## Implementation Order

1. `src/fzf.rs` — Core module with `FzfOptions`, `run_fzf()`, stdin detection
2. Refactor `source.rs` existing `--fzf` to use the new module
3. Add multi-select support to `list --fzf`
4. Add `equip fzf` top-level command
5. Wire up action bindings (ctrl-e for equip, etc.)
6. Add `--fzf` to `collect`, `equip`, `kit add`
7. Tests (unit tests for stdin detection, integration tests with fzf mock)
