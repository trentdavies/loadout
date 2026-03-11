## Why

Agent skills are manually duplicated across coding agents (Claude, Codex, Cursor, etc.). Each agent has its own directory structure and conventions, and keeping skills synchronized is tedious, error-prone, and doesn't scale. There is no tool to manage skills as a supply chain — sourcing them from marketplaces/repos, caching them locally, and installing them into the right agent targets in the right format.

## What Changes

- New Rust CLI (`skittle`) that manages the full lifecycle of agent skills across multiple coding agents
- Source management: register skill sources (git repos, local dirs, URLs), fetch/cache them locally in XDG-compliant paths
- Plugin system: sources contain plugins (the unit of packaging/distribution), plugins contain assets (skills now, MCPs/hooks/commands later)
- Local registry: cached, navigable by source, plugin, or skill. Skills follow the [Agent Skills specification](https://agentskills.io/specification)
- Target management: register agent targets (machine-global or repo-scoped), auto-detect installed agents, define custom target adapters via TOML
- Target adapters: each agent type has an adapter that transforms canonical Agent Skills format into the target's expected structure. Phase 1 ships `claude` and `codex` adapters (both passthrough)
- Bundles: user-defined groupings of skills across sources/plugins, installable to targets, swappable (clean replace)
- Progressive source detection: point skittle at a single SKILL.md, a directory, a plugin with plugin.toml, or a full multi-plugin source — it normalizes everything into the same internal model
- Skill identity: `source:plugin/skill` fully qualified, `plugin/skill` as default shorthand with disambiguation on collision
- TOML-driven configuration for everything: sources, targets, adapters, bundles, policies

## Capabilities

### New Capabilities
- `cli-framework`: Top-level CLI structure, global flags (-n, -v, -q, -h, --json, --color, --config), help system (help, -h, --help at every level)
- `source-management`: Register, remove, list, show, and update skill sources (file://, git://, github URLs). Fetch and cache sources locally
- `plugin-system`: Plugin as the unit of packaging. plugin.toml manifest. Source > Plugin > Asset hierarchy
- `source-detection`: Progressive detection algorithm — single file, flat dir, plugin dir (plugin.toml), full source (source.toml). Normalize all into consistent internal model
- `local-registry`: XDG-compliant local cache of all sources, plugins, and skills. Navigable by source, plugin, or skill
- `target-management`: Register, remove, list, show targets. Auto-detect agents on machine. Machine-global vs repo-scoped targets. Auto vs explicit sync modes
- `target-adapters`: TOML-defined adapters mapping skill format to target structure. Built-in `claude` and `codex` adapters (agentskills passthrough). Extensible format field for future converters
- `skill-operations`: List and show skills across the registry. Install/uninstall individual skills to targets
- `bundle-management`: Create, delete, list, show bundles. Add/drop skills. Install bundles to targets. Swap bundles (clean replace)
- `install-engine`: Unified install/uninstall verbs. --all, --skill, --plugin, --bundle, --target flags. -n dry run. Declarative convergence from config
- `config-management`: TOML config (sources, targets, adapters, bundles). config show + config edit. Cache management (clean, show)

### Modified Capabilities

(none — greenfield project)

## Impact

- New Rust binary crate, no existing code affected
- Dependencies: clap (CLI), toml/serde (config), reqwest (HTTP fetching), git2 (git operations), dirs (XDG paths)
- Creates files in XDG data/config dirs (~/.config/skittle/, ~/.local/share/skittle/)
- Writes into agent-specific directories (.claude/skills/, .codex/skills/, etc.)
- Reads from remote git repos and URLs
