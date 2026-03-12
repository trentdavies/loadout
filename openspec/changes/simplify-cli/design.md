## Context

The CLI has 30+ commands across 8 subgroups. Source resolution already collapses URL→source→plugin→skill automatically, making `source`, `plugin`, and `skill` subgroups redundant CLI surface. The core logic (fetch, detect, normalize, registry, adapters) is unaffected — this is purely a CLI surface reduction.

Key file: `src/cli/mod.rs` (~2000 lines) contains all command definitions and handlers in a single file.

## Goals / Non-Goals

**Goals:**
- Cut the CLI command surface from 30+ to ~18 commands
- Promote essential source operations (remove, update) to top-level
- Fold skill show into `list <name>`
- Update all tests (Rust + harness) and specs to match

**Non-Goals:**
- Changing any internal logic (source resolution, registry, install engine, adapters)
- Restructuring `src/cli/mod.rs` into multiple files
- Adding new functionality beyond the surface reshaping

## Decisions

### CLI surface is the only change layer
The `Command` enum and `run()` match arms in `src/cli/mod.rs` are the only code that changes. All `source::*`, `registry::*`, `target::*` modules remain untouched. The handler bodies for `remove` and `update` are copied from their `SourceCommand` counterparts — the logic is identical, just the dispatch path changes.

**Alternative considered:** Adding a compatibility layer that keeps old commands but marks them deprecated. Rejected — adds complexity for a tool with few users at this stage.

### `list <name>` folds in skill show
Rather than a separate `show` command, `list` accepts an optional positional argument. With no argument it lists all skills (table). With a name argument it shows details for that skill (key-value display). This mirrors common CLI patterns (e.g., `kubectl get` vs `kubectl get <name>`).

**Alternative considered:** `show <name>` as a separate top-level command. Rejected — `list`/`show` as separate commands adds surface area without value. The single `list` command covers both use cases.

### Filter flags (`--source`, `--plugin`) dropped from list
The current `skill list --source X --plugin Y` filters are removed. Users can grep the table output. These filters existed because the intermediate concepts had their own namespaces. With the simplified surface, filtering by internal concepts adds confusion.

### Removed specs deleted, not just emptied
The `source-management`, `plugin-system`, and `skill-operations` specs cover CLI commands that no longer exist. Rather than leaving empty specs, they're deleted. The underlying behavior (source resolution, plugin detection, skill validation) is covered by `source-detection` and `local-registry` specs which remain.

## Risks / Trade-offs

- **[Lost `source list`/`source show`]** → Users see sources via `status` or `config show`. Acceptable for current user base.
- **[Lost `cache clean`]** → `remove <name>` cleans per-source. No bulk clean. Users can `rm -rf ~/.local/share/skittle/sources/` if needed.
- **[Lost filter flags]** → `list | grep <pattern>` works. Acceptable tradeoff for simpler surface.
- **[Breaking change for existing users]** → All `skittle source add/remove/list/show/update`, `skittle plugin *`, `skittle skill *`, `skittle cache *` commands stop working. Mitigated by early stage of product.
