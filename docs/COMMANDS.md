# Loadout Command & Flag Reference

Agent skill manager — add, update, and install skills across coding agents.

---

## Global Flags

These flags are available on **every** command (`global = true`).

| Flag | Short | Type | Default | Behavior |
|------|-------|------|---------|----------|
| `--dry-run` | `-n` | bool | `false` | Show what would change without making modifications |
| `--verbose` | `-v` | bool | `false` | Verbose output — shows extra detail (e.g. skill paths, tree breakdowns) |
| `--quiet` | `-q` | bool | `false` | Suppress non-error output; skips interactive prompts (uses defaults) |
| `--json` | | bool | `false` | Output as JSON instead of human-readable text |
| `--config` | | `PATH` | None | Path to config file (overrides default config location) |

**Behavioral notes:**
- `--quiet` also suppresses interactive prompts and auto-accepts defaults (e.g. during `init`, `add`).
- `--json` switches output to `serde_json::to_string_pretty` for machine consumption.
- `--verbose` and `--quiet` are independent — `--quiet` wins for suppressing output, but `--verbose` can still add detail to error paths.

---

## Shorthand Syntax (Preprocessor)

Before clap parses args, `src/cli/args.rs` rewrites the raw arg vector.

### `@name` — Agent shorthand
- Expands to `--agent name`
- Active in: top-level equip (`_equip`), `agent collect`
- Elsewhere (e.g. `list`): passed through literally, no expansion

### `+name` — Kit shorthand
- Expands to `--kit name`
- Active in: top-level equip (`_equip`)
- In `agent collect`: **not expanded** (passed through as a positional)

### Top-level catch-all (equip)
When the first positional arg (after global flags) starts with `@` or `+`, the preprocessor injects the hidden `_equip` subcommand:

```
loadout @claude dev*        → loadout _equip dev* --agent claude
loadout +developer          → loadout _equip --kit developer
loadout -n @claude +dev *   → loadout -n _equip * --agent claude --kit dev
loadout @claude dev* --remove --force → loadout _equip dev* --remove --force --agent claude
```

There is no `loadout equip` or `loadout unequip` command. Equip is the default action when using `@`/`+` shorthand. Unequip is `--remove`.

### Equip flags

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--agent` | `-a` | string (repeatable) | None | Agent name(s) to target |
| `--all` | | bool | `false` | Target all configured agents |
| `--kit` | `-k` | string | None | Kit name to equip/unequip |
| `--save` | `-s` | bool | `false` | Save the resolved skill set as the kit given by `--kit` |
| `--force` | `-f` | bool | `false` | Overwrite changed skills without prompting (equip) / execute removal (unequip) |
| `--interactive` | `-i` | bool | `false` | Interactively resolve conflicts for changed skills |
| `--remove` | `-r` | bool | `false` | Unequip instead of equip |

**Flag conflicts:**
- `--agent` and `--all` conflict (clap-enforced).
- `--remove` cannot be combined with `--save` or `--interactive` (runtime validation).

**Agent resolution (when neither `--agent` nor `--all`):**
- Defaults to all agents with `sync: "auto"`.
- Errors if no agents are configured.

**`--kit` behavior:**
- When `--kit` is given without `--save`: loads the named kit and equips its skills.
- When `--kit` and `--save` are both given: resolves skills from patterns, saves them as the named kit, then equips.
- When `--save` is given without `--kit`: errors (kit name required).

**Conflict handling (equip only):**
- Default (no flag): skips changed skills, reports them.
- `--force`: overwrites without prompting.
- `--interactive`: prompts per-skill for changed files.

**Unequip behavior (`--remove`):**
- Without `--force`: preview mode — shows what would be removed but doesn't delete.
- With `--force`: actually removes skill files from agent directories.

**Examples:**
```
loadout @claude dev*                    # equip matching skills to claude
loadout @claude +dev                    # equip kit "dev" to claude
loadout +dev                            # equip kit to auto-sync agents
loadout @claude +dev --remove --force   # unequip kit from claude
loadout @claude dev* --remove --force   # unequip matching skills
loadout -n @claude +dev                 # dry-run equip
loadout @claude +newkit -s dev* -f      # equip and save as kit
```

### Expansion ordering
Shorthand tokens (`@`, `+`) are moved to **trailing position** as `--agent`/`--kit` flags, after all other positional args. This prevents clap's greedy `num_args` from consuming them.

### Quoting
`@"my-agent"` strips the surrounding quotes: expands to `--agent my-agent`.

### `--` (double dash)
Stops shorthand expansion for everything after it. Tokens after `--` are passed through literally.

---

## Commands

### `init`

Initialize loadout configuration.

```
loadout init [URL]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `url` | positional | no | Source URL to populate cache (GitHub URL or local path) |

**Behavior:**
- If config already exists: prints message and exits (no-op). If `url` was given, suggests `loadout add` instead.
- Creates directory structure: `data/`, `plugins/`, `cache/`, `.loadout/` (internal).
- Writes `.gitignore` (`external/`, `.loadout/`).
- Migrates legacy `sources/` → `external/` and `registry.json` → `.loadout/registry.json` if present.
- If `url` provided: fetches, detects, normalizes, and registers the source immediately.
- **Interactive wizard** (when not `--quiet` and stdin is a TTY):
  1. Prompts to `git init` the data dir (default: yes).
  2. Prompts to detect and add agents (scans `~/` and `./` for `.claude/`, `.codex/`, `.cursor/`).
  3. Offers popular skill marketplaces via multi-select (skipped if `url` was provided).
- In `--quiet` or non-interactive mode: auto-accepts all wizard defaults.

---

### `add`

Add a skill source.

```
loadout add <URL> [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `url` | positional | **yes** | URL or path to the source |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--source` | string | inferred from URL | Override the inferred source name |
| `--plugin` | string | inferred from structure | Override the inferred plugin name |
| `--skill` | string | inferred (single-skill sources only) | Override the inferred skill name |
| `--ref` | `REF` | None | Pin to a specific git ref (tag, branch, or commit SHA) |
| `--symlink` | bool | `false` | Symlink local directory sources instead of copying |
| `--copy` | bool | `false` | Copy local directory sources instead of symlinking |
| `--name` | string | — | **Deprecated** — errors with "renamed to `--source`" |

**Flag conflicts:**
- `--symlink` and `--copy` conflict (clap-enforced).

**Behavior:**
- Errors if source name already exists in config.
- For local directory sources: if neither `--symlink` nor `--copy` given, prompts for fetch mode (default: symlink).
- For git/remote sources: `--symlink`/`--copy` are ignored; always clones.
- If URL contains a tree ref (e.g. GitHub tree URL), uses it as the effective ref when `--ref` is not provided.
- In `--dry-run`: skips fetch, detection, and registration entirely.
- Without `--quiet`: prompts interactively to confirm source name, plugin name, and skill name (for single-skill sources).
- With `--verbose`: prints a tree of plugins and skills after adding.

---

### `list`

List skills, or show details for one.

```
loadout list [PATTERNS...] [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `patterns` | positional (variadic) | no | Skill identity or glob patterns (e.g. `plugin/skill`, `source:plugin/skill`, `legal/*`) |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--external` | bool | `false` | List external sources instead of skills |
| `--fzf` | bool | `false` | Interactive fuzzy finder with skill preview (requires `fzf` in PATH) |

**Behavior:**
- **No patterns, no flags:** Lists all skills (one per line, formatted identity).
- **With patterns:** Filters skills by exact match, glob, or freeform substring.
- **Single result:** Shows detail view (identity, description; path with `--verbose`).
- **`--external`:** Shows a table of configured sources (name, type, domain, ref, skill count, mode).
- **`--fzf`:** Pipes skills to `fzf` with SKILL.md preview. Prints selected identity to stdout. Errors if `fzf` not found.
- **`--json`:** Returns array of skill/source objects.

---

### `remove`

Remove a skill source.

```
loadout remove [NAME] [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | no | Source name (omit to select interactively) |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--force` | bool | `false` | Force removal even if skills are installed |

**Behavior:**
- Without `name`: prompts for interactive selection.
- Without `--force`: checks if skills from the source are installed and warns/blocks if so.

---

### `update`

Update source(s) from remote.

```
loadout update [NAME] [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | no | Source name (omit to update all) |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--ref` | `REF` | None | Switch to a specific git ref (tag or branch). Use `"latest"` to unpin. |

**Behavior:**
- Without `name`: updates all configured sources.
- `--ref latest`: removes the pinned ref, returning to default branch tracking.

---

### `status`

Show current status.

```
loadout status
```

No additional flags or arguments.

**Behavior:**
- Displays summary: source count (external/local breakdown), agent count (auto/explicit), total plugins, total skills, installed skills, kit count.
- With `--verbose`: shows per-source and per-agent detail with tree formatting.
- With `--json`: returns a JSON object with counts.

---

### `completions`

Generate shell completions.

```
loadout completions <SHELL> [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `shell` | positional (enum) | **yes** | One of: `bash`, `zsh`, `fish` |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--install` | bool | `false` | Auto-install to the standard location for your shell |

**Behavior:**
- Without `--install`: prints completion script to stdout.
- With `--install`: writes the completion script to the shell's standard completion directory.

---

### `_complete` (hidden)

Internal command used by shell completion scripts.

```
loadout _complete <KIND>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `kind` | positional | **yes** | Completion type: `sources`, `plugins`, `skills`, `agents`, `kits` |

---

### `config`

Manage configuration. Requires a subcommand.

#### `config show`

Show current configuration.

```
loadout config show
```

No additional flags or arguments.

#### `config edit`

Open config in editor.

```
loadout config edit
```

No additional flags or arguments.

---

### `kit`

Manage skill kits. Requires a subcommand.

#### `kit create`

Create a new kit, optionally seeding it with skills.

```
loadout kit create <NAME> [SKILLS...]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Kit name |
| `skills` | positional (variadic) | no | Skills or glob patterns to seed the kit with |

#### `kit delete`

Delete a kit.

```
loadout kit delete <NAME> [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Kit name |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--force` | bool | `false` | Force deletion |

#### `kit list`

List all kits, optionally filtered by name pattern.

```
loadout kit list [PATTERNS...]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `patterns` | positional (variadic) | no | Name patterns to filter by (glob supported) |

#### `kit show`

Show kit details.

```
loadout kit show <NAME>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Kit name |

#### `kit add`

Add skills to a kit.

```
loadout kit add <NAME> <SKILLS...>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Kit name |
| `skills` | positional (variadic) | **yes** | Skills to add (`plugin/skill`) |

#### `kit drop`

Remove skills from a kit.

```
loadout kit drop <NAME> <SKILLS...>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Kit name |
| `skills` | positional (variadic) | **yes** | Skills to remove (`plugin/skill`) |

---

### `agent`

Manage agents. Requires a subcommand.

#### `agent add`

Add an agent.

```
loadout agent add <AGENT_TYPE> [PATH] [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `agent` | positional | **yes** | Agent type (`claude`, `codex`, `cursor`, etc.) |
| `path` | positional | no | Path to agent directory |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--name` | string | None | Name for this agent (inferred from path if omitted) |
| `--scope` | string | `"machine"` | Scope: `machine` or `repo` |
| `--sync` | string | `"auto"` | Sync mode: `auto` or `explicit` |

**Behavior:**
- `--scope machine`: agent lives in home dir (e.g. `~/.claude`).
- `--scope repo`: agent lives in project dir.
- `--sync auto`: agent is included by default when no `--agent` flag is given to `equip`/`unequip`.
- `--sync explicit`: agent is only targeted when explicitly named via `--agent`.

#### `agent remove`

Remove an agent.

```
loadout agent remove <NAME> [flags]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Agent name |

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--force` | bool | `false` | Actually perform the removal (default is dry run / preview) |

#### `agent list`

List all agents.

```
loadout agent list
```

No additional flags or arguments.

#### `agent show`

Show agent details.

```
loadout agent show <NAME>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | positional | **yes** | Agent name |

#### `agent detect`

Detect agent installations and prompt to add them.

```
loadout agent detect [flags]
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--force` | bool | `false` | Automatically add all detected agents without prompting |

**Behavior:**
- Scans `~/` and `./` for directories matching `.claude`, `.codex`, `.cursor` (and variants like `.claude-*`).
- Without `--force`: prompts per agent.
- With `--force`: adds all detected agents silently.
- Scope is auto-detected: home dir → `machine`, cwd → `repo`. Repo-scoped agents default to `sync: explicit`.

#### `agent collect`

Collect skills from an agent back to source.

```
loadout agent collect [flags]
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--agent` | `AGENT` | **required** | Agent to collect from |
| `--skill` | `SKILL` | None | Skill name to collect (omit for all) |
| `--adopt` | bool | `false` | Adopt skill into `plugins/` (make it yours) |
| `--force` | bool | `false` | Auto-adopt all untracked skills without prompting |

**Behavior:**
- Reads skills from the agent's installed directory and syncs them back to the source cache.
- `--adopt`: copies the skill into the local `plugins/` directory, making it a local (non-external) skill.
- `--force`: skips the adoption prompt for untracked skills.
- Without `--skill`: processes all skills found in the agent.

**Shorthand:**
- `@name` expands to `--agent name` in this subcommand.
- `+name` is **not** expanded in `collect` (only `@` is).
