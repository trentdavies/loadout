# Loadout

Agent skill manager — source, cache, and install skills across coding agents.

Loadout gives you a single supply chain for agent skills. Add sources from GitHub repos, local directories, or archives. Organize skills into bundles. Install them to Claude, Codex, Cursor, or any agent that reads structured skill files. The skills are just markdown (the [Agent Skills](https://agentskills.io) spec), so your entire skill library is version-controllable, diffable, and portable.

## Why

Coding agents are accumulating skills fast. A team with 5 agents across 3 projects can easily have 50+ skills scattered across `~/.claude/`, `~/.codex/`, project-level configs, and random repos. There's no way to keep them in sync, no way to swap contexts (i.e., "work mode" vs "personal mode"), and no provenance tracking when someone edits a skill at the target.

Loadout treats this as a package management problem: sources provide skills, targets consume them, and the registry tracks what came from where.

## How It Works

```
Sources (GitHub, local dirs, archives)
    │
    ▼
┌──────────────────────────────────────┐
│  ~/.local/share/loadout/             │
│  ├── plugins/        (your skills)   │
│  ├── external/       (cached sources)│
│  ├── loadout.toml    (config)        │
│  └── .loadout/                       │
│      └── registry.json (provenance)  │
└──────────────────────────────────────┘
    │
    ▼
Targets (~/.claude, ~/.codex, ./project/.cursor, etc.)
```

Your `~/.local/share/loadout/` directory is itself a valid skill marketplace. We recommend managing it with git — `plugins/` contains your authored skills, `loadout.toml` tracks your sources and targets, and `external/` is gitignored cache. Push it to GitHub and you have a portable, versioned skill library that follows you across machines.

Then layer in additional marketplaces from teammates, the community, or your organization:

```bash
loadout add https://github.com/your-org/agent-skills
loadout add https://github.com/anthropic/claude-plugins --ref v2.1
loadout add ~/dev/my-local-skills --symlink
```

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize loadout
loadout init <your-personal-skills-marketplace>
loadout init # interactive onboard

# Add a skill source (GitHub repo, local dir, or archive)
loadout add https://github.com/your-org/agent-skills

# Register your agent targets
loadout target detect          # auto-detect Claude, Codex, etc.
loadout target add claude ~/.claude --name claude-global

# See what's available
loadout list

# Install everything
loadout apply --all

# Check status
loadout status
```

## Core Concepts

**Skills** follow the [Agent Skills](https://agentskills.io) spec: a directory with a `SKILL.md` file containing YAML frontmatter (`name`, `description`) and optional `scripts/`, `references/`, `assets/` directories.

**Sources** are where skills come from: git repos, local directories, single files, or zip archives. Each source can contain multiple plugins, each containing multiple skills.

**Targets** are where skills get installed: `~/.claude`, `~/.codex`, project-level agent configs, or custom paths with custom adapters.

**Bundles** are named groups of skills you can activate/deactivate as a unit. Think "work mode" vs "personal projects" vs "writing."

**Identity** — every skill has a fully qualified identity: `source:plugin/skill`. Short form `plugin/skill` works when unambiguous.

## Common Workflows

### Adding Sources

`loadout add` accepts just about anything and tries to do the right thing. It detects the source type, scans the structure, and infers source/plugin/skill names automatically.

```bash
# GitHub repo — full HTTPS URL
loadout add https://github.com/acme/agent-skills

# GitHub shorthand — org/repo expands to https://github.com/org/repo.git
loadout add acme/agent-skills

# Git SSH
loadout add git@github.com:acme/agent-skills.git

# Pin to a specific tag, branch, or commit
loadout add acme/agent-skills --ref v2.0

# Local directory (symlinked by default — your edits are live)
loadout add ~/dev/my-skills
loadout add ./relative/path
loadout add /absolute/path

# Local directory as a copied snapshot
loadout add ~/dev/my-skills --copy

# Zip archive — extracted and cached
loadout add ./plugins.zip

# .skill archive — single skill packaged for sharing
loadout add ./code-review.skill

# Override inferred names
loadout add acme/tools --source acme --plugin devtools
```

Loadout auto-detects the structure of whatever you point it at:

| What you add | What loadout sees | Result |
|---|---|---|
| Repo with `marketplace.json` | Full marketplace | Multiple plugins, each with skills |
| Repo with `.claude-plugin/` | Single plugin | One plugin with its skills |
| Directory with subdirs containing `SKILL.md` | Flat skills | Auto-wraps them in an implicit plugin |
| Single directory with `SKILL.md` | Single skill | Auto-creates plugin and skill |
| Single `.md` file with frontmatter | Single file | Auto-creates plugin, skill, and directory |

When running interactively (TTY), loadout confirms inferred names and lets you override them. In scripts or CI, it uses the inferred defaults silently (or pass `--quiet`).

### Discovering Skills

```bash
# List everything
loadout list

# Filter with glob patterns
loadout list "legal/*"                 # all skills in "legal" plugin
loadout list "acme:*/*"                # everything from the "acme" source
loadout list "*/code-*" "*/debug-*"    # multiple patterns (union)

# Show details for one skill
loadout list legal/contract-review

# List external sources
loadout list --external

# JSON output for scripting
loadout list --json
```

### Installing Skills

```bash
# Apply all skills to all auto-sync targets
loadout apply --all

# Apply a specific skill
loadout apply --skill legal/contract-review

# Apply all skills from a plugin
loadout apply --plugin legal

# Apply to a specific target only
loadout apply --all --target claude-global

# Interactive conflict resolution (skip/overwrite/diff per skill)
loadout apply --all -i

# Force overwrite changed skills
loadout apply --all --force

# Preview what would happen
loadout apply --all --dry-run
```

### Managing Bundles

Bundles let you swap skill contexts in one command.

```bash
# Create bundles
loadout bundle create work
loadout bundle create personal

# Add skills (glob patterns supported)
loadout bundle add work "legal/*" "engineering/*"
loadout bundle add personal "writing/*" "productivity/*"

# Activate a bundle (batch install)
loadout bundle activate work --all --force

# Switch contexts
loadout bundle deactivate work --all --force
loadout bundle activate personal --all --force

# See what's configured
loadout bundle list
loadout bundle show work
```

### Updating Sources

```bash
# Update all sources
loadout update

# Update one source
loadout update acme

# Switch to a different version
loadout update acme --ref v3.0

# Unpin and track latest
loadout update acme --ref latest
```

### Collecting Edits

When someone edits a skill directly at the target (e.g., tweaks a prompt in `~/.claude/skills/`), `collect` brings those changes back:

```bash
# Collect modified skills from a target
loadout collect --target claude-global

# Collect a specific skill
loadout collect --skill code-review --target claude-global

# Adopt untracked skills into your local plugins
loadout collect --target claude-global --adopt
```

### Managing Targets

```bash
# Auto-detect agent installations
loadout target detect

# Add manually
loadout target add claude ~/.claude --name claude-global
loadout target add codex ~/.codex --name codex-global
loadout target add claude ./project/.claude --name project-claude

# List targets
loadout target list

# Remove a target
loadout target remove project-claude
```

## Directory Layout

```
~/.local/share/loadout/
├── loadout.toml                  # Sources, targets, bundles, adapters
├── .claude-plugin/
│   └── marketplace.json          # Auto-generated from plugins/
├── .loadout/
│   └── registry.json             # Provenance tracking
├── plugins/                      # Your skills (git-tracked)
│   └── my-tools/
│       └── skills/
│           ├── code-review/
│           │   └── SKILL.md
│           └── debug-helper/
│               └── SKILL.md
└── external/                     # Cached sources (gitignored)
    ├── acme-skills/
    └── community-plugins/
```

## Custom Adapters

Built-in adapters handle Claude and Codex. For other agents, define custom adapters in `loadout.toml`:

```toml
[adapter.my-agent]
skill_dir = "prompts/{name}"
skill_file = "prompt.md"
format = "agentskills"
copy_dirs = ["scripts", "references"]
```

Then register a target using your adapter:

```bash
loadout target add my-agent ~/.my-agent --name my-agent-global
```

## Global Flags

Every command supports:

| Flag | Description |
|------|-------------|
| `-n`, `--dry-run` | Preview changes without modifying anything |
| `-v`, `--verbose` | Detailed output |
| `-q`, `--quiet` | Suppress non-error output |
| `--json` | Machine-readable JSON output |
| `--config <path>` | Override config file location |

## Development

```bash
just build       # Debug build
just test        # Unit + integration tests
just check       # Clippy + fmt
just fix         # Auto-fix clippy + fmt
just harness     # Docker-based functional tests
just release     # Release build
```

## License

MIT
