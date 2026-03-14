# Skittle

Agent skill manager — source, cache, and install skills across coding agents.

Skittle gives you a single supply chain for agent skills. Add sources from GitHub repos, local directories, or archives. Organize skills into bundles. Install them to Claude, Codex, Cursor, or any agent that reads structured skill files. The skills are just markdown (the [Agent Skills](https://agentskills.io) spec), so your entire skill library is version-controllable, diffable, and portable.

## Why

Coding agents are accumulating skills fast. A team with 5 agents across 3 projects can easily have 50+ skills scattered across `~/.claude/`, `~/.codex/`, project-level configs, and random repos. There's no way to keep them in sync, no way to swap contexts (i.e., "work mode" vs "personal mode"), and no provenance tracking when someone edits a skill at the target.

Skittle treats this as a package management problem: sources provide skills, targets consume them, and the registry tracks what came from where.

## How It Works

```
Sources (GitHub, local dirs, archives)
    │
    ▼
┌──────────────────────────────────────┐
│  ~/.local/share/skittle/             │
│  ├── plugins/        (your skills)   │
│  ├── external/       (cached sources)│
│  ├── skittle.toml    (config)        │
│  └── .skittle/                       │
│      └── registry.json (provenance)  │
└──────────────────────────────────────┘
    │
    ▼
Targets (~/.claude, ~/.codex, ./project/.cursor, etc.)
```

Your `~/.local/share/skittle/` directory is itself a valid skill marketplace. We recommend managing it with git — `plugins/` contains your authored skills, `skittle.toml` tracks your sources and targets, and `external/` is gitignored cache. Push it to GitHub and you have a portable, versioned skill library that follows you across machines.

Then layer in additional marketplaces from teammates, the community, or your organization:

```bash
skittle add https://github.com/your-org/agent-skills
skittle add https://github.com/anthropic/claude-plugins --ref v2.1
skittle add ~/dev/my-local-skills --symlink
```

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize skittle
skittle init <your-personal-skills-marketplace>
skittle init # interactive onboard

# Add a skill source (GitHub repo, local dir, or archive)
skittle add https://github.com/your-org/agent-skills

# Register your agent targets
skittle target detect          # auto-detect Claude, Codex, etc.
skittle target add claude ~/.claude --name claude-global

# See what's available
skittle list

# Install everything
skittle apply --all

# Check status
skittle status
```

## Core Concepts

**Skills** follow the [Agent Skills](https://agentskills.io) spec: a directory with a `SKILL.md` file containing YAML frontmatter (`name`, `description`) and optional `scripts/`, `references/`, `assets/` directories.

**Sources** are where skills come from: git repos, local directories, single files, or zip archives. Each source can contain multiple plugins, each containing multiple skills.

**Targets** are where skills get installed: `~/.claude`, `~/.codex`, project-level agent configs, or custom paths with custom adapters.

**Bundles** are named groups of skills you can activate/deactivate as a unit. Think "work mode" vs "personal projects" vs "writing."

**Identity** — every skill has a fully qualified identity: `source:plugin/skill`. Short form `plugin/skill` works when unambiguous.

## Common Workflows

### Adding Sources

`skittle add` accepts just about anything and tries to do the right thing. It detects the source type, scans the structure, and infers source/plugin/skill names automatically.

```bash
# GitHub repo — full HTTPS URL
skittle add https://github.com/acme/agent-skills

# GitHub shorthand — org/repo expands to https://github.com/org/repo.git
skittle add acme/agent-skills

# Git SSH
skittle add git@github.com:acme/agent-skills.git

# Pin to a specific tag, branch, or commit
skittle add acme/agent-skills --ref v2.0

# Local directory (symlinked by default — your edits are live)
skittle add ~/dev/my-skills
skittle add ./relative/path
skittle add /absolute/path

# Local directory as a copied snapshot
skittle add ~/dev/my-skills --copy

# Zip archive — extracted and cached
skittle add ./plugins.zip

# .skill archive — single skill packaged for sharing
skittle add ./code-review.skill

# Override inferred names
skittle add acme/tools --source acme --plugin devtools
```

Skittle auto-detects the structure of whatever you point it at:

| What you add | What skittle sees | Result |
|---|---|---|
| Repo with `marketplace.json` | Full marketplace | Multiple plugins, each with skills |
| Repo with `.claude-plugin/` | Single plugin | One plugin with its skills |
| Directory with subdirs containing `SKILL.md` | Flat skills | Auto-wraps them in an implicit plugin |
| Single directory with `SKILL.md` | Single skill | Auto-creates plugin and skill |
| Single `.md` file with frontmatter | Single file | Auto-creates plugin, skill, and directory |

When running interactively (TTY), skittle confirms inferred names and lets you override them. In scripts or CI, it uses the inferred defaults silently (or pass `--quiet`).

### Discovering Skills

```bash
# List everything
skittle list

# Filter with glob patterns
skittle list "legal/*"                 # all skills in "legal" plugin
skittle list "acme:*/*"                # everything from the "acme" source
skittle list "*/code-*" "*/debug-*"    # multiple patterns (union)

# Show details for one skill
skittle list legal/contract-review

# List external sources
skittle list --external

# JSON output for scripting
skittle list --json
```

### Installing Skills

```bash
# Apply all skills to all auto-sync targets
skittle apply --all

# Apply a specific skill
skittle apply --skill legal/contract-review

# Apply all skills from a plugin
skittle apply --plugin legal

# Apply to a specific target only
skittle apply --all --target claude-global

# Interactive conflict resolution (skip/overwrite/diff per skill)
skittle apply --all -i

# Force overwrite changed skills
skittle apply --all --force

# Preview what would happen
skittle apply --all --dry-run
```

### Managing Bundles

Bundles let you swap skill contexts in one command.

```bash
# Create bundles
skittle bundle create work
skittle bundle create personal

# Add skills (glob patterns supported)
skittle bundle add work "legal/*" "engineering/*"
skittle bundle add personal "writing/*" "productivity/*"

# Activate a bundle (batch install)
skittle bundle activate work --all --force

# Switch contexts
skittle bundle deactivate work --all --force
skittle bundle activate personal --all --force

# See what's configured
skittle bundle list
skittle bundle show work
```

### Updating Sources

```bash
# Update all sources
skittle update

# Update one source
skittle update acme

# Switch to a different version
skittle update acme --ref v3.0

# Unpin and track latest
skittle update acme --ref latest
```

### Collecting Edits

When someone edits a skill directly at the target (e.g., tweaks a prompt in `~/.claude/skills/`), `collect` brings those changes back:

```bash
# Collect modified skills from a target
skittle collect --target claude-global

# Collect a specific skill
skittle collect --skill code-review --target claude-global

# Adopt untracked skills into your local plugins
skittle collect --target claude-global --adopt
```

### Managing Targets

```bash
# Auto-detect agent installations
skittle target detect

# Add manually
skittle target add claude ~/.claude --name claude-global
skittle target add codex ~/.codex --name codex-global
skittle target add claude ./project/.claude --name project-claude

# List targets
skittle target list

# Remove a target
skittle target remove project-claude
```

## Directory Layout

```
~/.local/share/skittle/
├── skittle.toml                  # Sources, targets, bundles, adapters
├── .claude-plugin/
│   └── marketplace.json          # Auto-generated from plugins/
├── .skittle/
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

Built-in adapters handle Claude and Codex. For other agents, define custom adapters in `skittle.toml`:

```toml
[adapter.my-agent]
skill_dir = "prompts/{name}"
skill_file = "prompt.md"
format = "agentskills"
copy_dirs = ["scripts", "references"]
```

Then register a target using your adapter:

```bash
skittle target add my-agent ~/.my-agent --name my-agent-global
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
