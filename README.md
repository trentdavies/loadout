# Loadout

Loadout manages skills across coding agents. You maintain a central skill library, and loadout handles syncing skills to `~/.claude`, `~/.codex`, `./project/.cursor`, and any other agent directory on your machine.

## Install

```bash
cargo install --path .
```

## Setup

```bash
loadout init
```

This detects your agents (Claude, Codex, Cursor), offers popular skill sources, and creates your skill library at `~/.local/share/loadout/`. The library is a git-friendly directory you can push to a repo and share across machines.

## Add Sources

Sources provide skills. Point loadout at a GitHub repo, local directory, or archive and it figures out the structure.

```bash
loadout add https://github.com/acme/agent-skills
loadout add acme/agent-skills                      # GitHub shorthand
loadout add ~/dev/my-skills                        # local dir (symlinked)
loadout add ~/dev/my-skills --copy                 # local dir (snapshot)
loadout add acme/agent-skills --ref v2.0           # pinned to a tag
```

See what you have:

```bash
loadout list                    # all skills
loadout list "legal/*"          # glob filter
loadout list --external         # list sources
loadout status                  # overview
```

## Equip Skills

Pick agents with `@`, pick a kit with `+`, and pass skill patterns as arguments.

```bash
# Equip skills to Claude
loadout @claude "legal/*" "productivity/start" -f

# Equip a kit
loadout @claude +developer -f

# Kit plus extra skills
loadout @claude +developer "legal/*" -f

# Multiple agents
loadout @claude @codex +developer -f

# Omit @agent to target all auto-sync agents
loadout +developer -f

# Preview first
loadout -n @claude +developer

# Interactive conflict resolution
loadout @claude +developer -i

# Unequip
loadout @claude +developer -r -f
```

| Flag | Short | Description |
|------|-------|-------------|
| `--force` | `-f` | Overwrite changed skills / execute removal |
| `--interactive` | `-i` | Resolve conflicts per-skill (skip, overwrite, diff) |
| `--save` | `-s` | Save resolved skills as a kit (`loadout @claude +new-kit -s "dev*" -f`) |
| `--remove` | `-r` | Unequip instead of equip |

## Collect Edits

When someone tweaks a skill directly at the agent, collect brings changes back to your library.

```bash
loadout agent collect --agent claude                     # show tracked vs untracked
loadout agent collect --agent claude --skill code-review # collect one skill
loadout agent collect --agent claude --adopt             # adopt untracked skills
```

## Kits

Kits are named skill sets you equip and unequip as a unit.

```bash
loadout kit create developer "dev*" "productivity/*"
loadout kit add developer "legal/contract-review"
loadout kit drop developer "productivity/start"
loadout kit list
loadout kit show developer
```

Or create on-the-fly during equip:

```bash
loadout @claude +new-kit -s "dev*" "legal/*" -f
```

## Agents

```bash
loadout agent detect                                          # auto-detect
loadout agent add claude ~/.claude --name claude-global       # manual
loadout agent add claude ./.claude --name project --scope repo
loadout agent list
loadout agent show claude-global
loadout agent remove project --force
```

Agents have a **scope** (machine or repo) and a **sync mode** (auto or explicit). Auto-sync agents are targeted by default when you omit `@agent`. Repo-scoped agents default to explicit sync.

## Updating Sources

```bash
loadout update              # all sources
loadout update acme         # one source
loadout update acme --ref v3.0
loadout update acme --ref latest   # unpin
```

## Custom Adapters

Built-in adapters cover Claude, Codex, Cursor, Gemini, and VS Code. Define your own in `loadout.toml`:

```toml
[adapter.my-agent]
skill_dir = "prompts/{name}"
skill_file = "prompt.md"
format = "agentskills"
copy_dirs = ["scripts", "references"]
```

## Global Flags

Every command supports these:

| Flag | Description |
|------|-------------|
| `-n`, `--dry-run` | Preview without changes |
| `-v`, `--verbose` | Detailed output |
| `-q`, `--quiet` | Suppress non-error output |
| `--json` | Machine-readable output |
| `--config <path>` | Override config location |

## Development

```bash
just test           # unit + integration tests
just harness        # Docker offline tests
just sandbox-test   # Docker tests against real repos
just check          # clippy + fmt
```

## License

MIT
