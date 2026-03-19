# Equip

Equip manages skills across coding agents. You maintain a central skill library, and equip handles syncing skills to `~/.claude`, `~/.codex`, `./project/.cursor`, and any other agent directory on your machine.

## Install

```bash
brew tap trentdavies/tap
brew install equip
```

Or with Cargo:

```bash
cargo install equip
```

## Setup

```bash
equip init
```

This detects your agents (Claude, Codex, Cursor) and creates your skill library at `~/.local/share/equip/`. The library is a git-friendly directory you can push to a repo and share across machines.

## Add Sources

Sources provide skills. Point equip at a GitHub repo, local directory, or archive and it figures out the structure.

```bash
equip add https://github.com/acme/agent-skills
equip add acme/agent-skills                      # GitHub shorthand
equip add ~/dev/my-skills                        # local dir (symlinked)
equip add ~/dev/my-skills --copy                 # local dir (snapshot)
equip add acme/agent-skills --ref v2.0           # pinned to a tag
```

See what you have:

```bash
equip list                    # all skills
equip list "legal/*"          # glob filter
equip source list             # list sources
equip list --external         # compatibility alias for source list
equip status                  # overview
```

Remove what you own:

```bash
equip remove test-plugin/explore   # remove a local skill
equip remove test-plugin/*         # remove matching local skills
equip source remove acme           # remove a source explicitly
```

## Equip Skills

Pick agents with `@`, pick a kit with `+`, and pass skill patterns as arguments.

```bash
# Equip skills to Claude
equip @claude "legal/*" "productivity/start" -f

# Equip a kit
equip @claude +developer -f

# Kit plus extra skills
equip @claude +developer "legal/*" -f

# Multiple agents
equip @claude @codex +developer -f

# Omit @agent to target all auto-sync agents
equip +developer -f

# Preview first
equip -n @claude +developer

# Interactive conflict resolution
equip @claude +developer -i

# Unequip
equip @claude +developer -r -f
```

| Flag | Short | Description |
|------|-------|-------------|
| `--force` | `-f` | Overwrite changed skills / execute removal |
| `--interactive` | `-i` | Resolve conflicts per-skill (skip, overwrite, diff) |
| `--save` | `-s` | Save resolved skills as a kit (`equip @claude +new-kit -s "dev*" -f`) |
| `--remove` | `-r` | Unequip instead of equip |

## Collect Edits

When someone tweaks a skill directly at the agent, collect brings changes back to your library.

```bash
equip collect --agent claude                # show tracked vs untracked
equip collect @claude code-review           # collect one skill
equip collect @claude +developer            # collect a maintained kit
equip collect --agent claude --adopt-local  # adopt untracked skills
equip collect @claude stray-skill --link my-src:plugin/skill
equip agent collect --agent claude          # compatibility alias
```

## Kits

Kits are named skill sets you equip and unequip as a unit.

```bash
equip kit create developer "dev*" "productivity/*"
equip kit add developer "legal/contract-review"
equip kit drop developer "productivity/start"
equip kit list
equip kit show developer
```

Or create on-the-fly during equip:

```bash
equip @claude +new-kit -s "dev*" "legal/*" -f
```

## Agents

```bash
equip agent detect                                          # auto-detect
equip agent add claude ~/.claude --name claude-global       # manual
equip agent add claude ./.claude --name project --scope repo
equip agent list
equip agent show claude-global
equip agent remove project --force
```

Agents have a **scope** (machine or repo) and a **sync mode** (auto or explicit). Auto-sync agents are targeted by default when you omit `@agent`. Repo-scoped agents default to explicit sync.

## Updating Sources

```bash
equip source update              # all sources
equip source update acme         # one source
equip source update acme --ref v3.0
equip source update acme --ref latest   # unpin
```

## Custom Adapters

Built-in adapters cover Claude, Codex, Cursor, Gemini, and VS Code. Define your own in `equip.toml`:

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
