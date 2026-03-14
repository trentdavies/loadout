## Why

`skittle add` silently infers up to three names from a URL — source (from the URL), plugin (from manifest or directory structure), and skill (from skill directories) — then registers them without confirmation. The user has no visibility into what identities will be created. `skittle remove` requires the user to already know the inferred source name. Both commands should surface the detected names and let the user confirm or override them interactively.

## What Changes

- `add`: After fetching and detecting structure, show the resolved identity hierarchy (source → plugin → skill) and prompt the user to confirm or override each level. The prompt is bypassed when explicit flags are passed:
  - `--source` overrides the inferred source name (replaces current `--name`)
  - `--plugin` overrides the inferred plugin name
  - `--skill` overrides the inferred skill name (only meaningful for single-skill sources)
  - When all applicable flags are provided, no prompt is shown.
  - When `--quiet` is passed or stdin is not a TTY, use inferred defaults without prompting.
- `remove`: When called without a source name argument, list available sources and prompt the user to select one. Current behavior (positional name arg) continues to bypass the prompt.
- Add a shared interactive prompting utility that handles confirm-with-default, select-from-list, and TTY detection.

### Resolution rules (for context)

The identity hierarchy `source:plugin/skill` is inferred differently depending on the detected source structure:

| Structure | Source name | Plugin name | Skill name |
|---|---|---|---|
| SingleFile | from URL | = source name | filename stem |
| SingleSkillDir | from URL | = source name | directory name |
| SinglePlugin | from URL | from manifest or dir name | from skill dirs |
| FlatSkills | from URL | cache directory name | from skill dirs |
| Marketplace | from URL | from marketplace.json | from skill dirs |

When plugin name = source name (SingleFile, SingleSkillDir), prompting for plugin is redundant and should be skipped. The `--plugin` flag is only relevant when the structure produces a distinct plugin name.

## Capabilities

### New Capabilities
- `interactive-prompts`: Shared TTY-aware prompting for CLI commands — confirm/override defaults, select from lists, respect `--quiet` and non-TTY contexts.

### Modified Capabilities
- `cli-framework`: `add` gains interactive confirmation of resolved identities with `--source`, `--plugin`, `--skill` override flags (replacing `--name`). **BREAKING**: `--name` renamed to `--source`. `remove` gains interactive source-selection when no positional arg is provided.

## Impact

- `src/cli/mod.rs`: `Command::Add` struct changes (`--name` → `--source`, add `--plugin`/`--skill`). Add and Remove handlers gain prompt logic after fetch/detect.
- `src/source/normalize.rs`: Normalize must accept optional overrides for plugin/skill names.
- New module for prompt utilities (TTY detection, confirm-with-default, list selection).
- `Cargo.toml`: May need a TTY/prompt dependency (e.g., `dialoguer` or raw `isatty` check).
- Non-interactive contexts (CI, piped input): Use inferred defaults silently — never block.
