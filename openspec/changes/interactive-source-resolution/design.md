## Context

`skittle add` and `skittle remove` operate non-interactively. The `add` command infers source/plugin/skill names through a chain of detection (`detect`) and normalization (`normalize`) — but the user never sees or confirms these inferred names until after registration. The `remove` command requires the user to already know the source name.

The identity hierarchy `source:plugin/skill` has different inference rules depending on source structure (SingleFile, SinglePlugin, FlatSkills, Marketplace, SingleSkillDir). Sometimes plugin name = source name, sometimes it comes from a manifest, sometimes from a directory name. This is opaque to the user.

## Goals / Non-Goals

**Goals:**
- Surface inferred identities to the user before committing them
- Let users override any level (source, plugin, skill) via flags for scripting
- Degrade gracefully in non-TTY / `--quiet` contexts (use defaults, don't block)

**Non-Goals:**
- Changing how detection or normalization works internally
- Adding prompts to any other commands (apply, update, etc.)
- Interactive editing of skill content or metadata

## Decisions

### 1. Prompt module using `dialoguer`

Add `dialoguer` as a dependency for TTY-aware prompts. It handles raw terminal input, default values, and selection lists. The alternative — hand-rolling stdin reads — is fragile around edge cases (line buffering, signal handling, Windows compatibility).

A `prompt` module in `src/prompt/` exposes:
- `confirm_or_override(label, default) -> String` — shows the default, lets user press Enter to accept or type a replacement
- `select_from(label, options) -> String` — numbered list selection
- `is_interactive() -> bool` — checks stdin isatty

### 2. Prompt placement in `add`: after fetch+detect, before normalize+save

The prompt fires after `fetch` and `detect` complete (so we know the structure and defaults) but before `normalize` and registry save. This is the natural seam — we have the inferred names but haven't committed them yet.

Flow:
1. Parse URL → `source_url`
2. Derive default source name → `source_url.default_name()`
3. **If `--source` not passed and interactive**: prompt for source name with default
4. Fetch into cache
5. Detect structure
6. Derive default plugin/skill names from structure
7. **If `--plugin`/`--skill` not passed and interactive**: prompt for overrides (skip if plugin = source for SingleFile/SingleSkillDir)
8. Normalize with final names
9. Save to registry + config

Alternative considered: prompt before fetch. Rejected because we can't show plugin/skill defaults until we've fetched and detected the structure.

### 3. Flag naming: `--source`, `--plugin`, `--skill`

Short, unambiguous, and match the identity hierarchy. `--name` is removed (breaking). The flags map directly to the three levels of `source:plugin/skill`.

For `remove`, no new flags — the positional arg already serves as the non-interactive path. The interactive path is triggered by omitting the positional.

### 4. Non-interactive fallback

When `is_interactive()` returns false (piped stdin, CI, `--quiet`):
- Use inferred defaults without prompting
- Print the resolved identities to stderr (unless `--quiet`) so the user can see what was chosen
- Never block on stdin

## Risks / Trade-offs

- **New dependency (`dialoguer`)**: Adds compile-time cost. Mitigation: it's widely used, well-maintained, and avoids hand-rolling terminal handling.
- **Breaking `--name` → `--source`**: Existing scripts using `--name` will break. Mitigation: clear error message suggesting `--source` if `--name` is passed.
- **Prompt ordering**: Source name is prompted before fetch (needed for cache path), but plugin/skill are prompted after (need structure detection). This means the user sees two prompt phases. Mitigation: keep each phase brief (one line each).
