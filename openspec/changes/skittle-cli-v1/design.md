## Context

This is a greenfield Rust CLI project. No existing code. The project manages agent skills across coding agents (Claude, Codex, Cursor, Gemini, etc.) — sourcing skills from marketplaces/repos, caching them in a local registry, and installing them into agent-specific target directories.

Skills follow the [Agent Skills specification](https://agentskills.io/specification): a directory containing `SKILL.md` (YAML frontmatter + markdown instructions) plus optional `scripts/`, `references/`, and `assets/` directories.

The flow is one-directional: Sources → Local Registry → Targets. Configuration is TOML-driven.

## Goals / Non-Goals

**Goals:**
- Single binary CLI that manages the full skill lifecycle across agents
- Support multiple source types (local filesystem, git repos, URLs)
- Progressive source detection (single file through multi-plugin marketplace)
- Extensible target adapter system (TOML-defined, built-in claude + codex for phase 1)
- Bundle system for grouping and swapping skill sets per workflow context
- Elite CLI UX: consistent flags, help at every level, dry-run, JSON output, colored output

**Non-Goals:**
- GUI or TUI interface
- Non-skill asset types (MCPs, hooks, commands) — deferred to future phases
- Bidirectional sync (editing installed skills and propagating back)
- Format conversion adapters beyond passthrough (mdc, json converters are future)
- Marketplace search/discovery APIs
- Remote targets (pushing to GitHub repos) — deferred
- Plugin authoring tools (scaffolding, validation) — use agentskills tooling

## Decisions

### Crate structure: single binary crate, library-driven

The project is a single Cargo workspace with one binary crate. All logic lives in a `lib.rs`-rooted library, with `main.rs` as a thin CLI entry point. This enables future testing and potential library reuse without premature extraction into multiple crates.

```
skittle/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, clap parse + dispatch
│   ├── lib.rs               # Public API
│   ├── cli/                 # Command definitions + handlers
│   │   ├── mod.rs
│   │   ├── install.rs
│   │   ├── uninstall.rs
│   │   ├── source.rs
│   │   ├── plugin.rs
│   │   ├── skill.rs
│   │   ├── bundle.rs
│   │   ├── target.rs
│   │   ├── config.rs
│   │   ├── cache.rs
│   │   ├── status.rs
│   │   └── init.rs
│   ├── registry/            # Local registry + cache
│   │   ├── mod.rs
│   │   ├── source.rs
│   │   ├── plugin.rs
│   │   ├── skill.rs
│   │   └── cache.rs
│   ├── source/              # Source fetching + detection
│   │   ├── mod.rs
│   │   ├── fetch.rs         # URL resolution, git clone, file copy
│   │   ├── detect.rs        # Progressive structure detection
│   │   └── normalize.rs     # Normalize any source shape into registry model
│   ├── target/              # Target management + adapters
│   │   ├── mod.rs
│   │   ├── adapter.rs       # Adapter trait + TOML-driven adapter
│   │   ├── detect.rs        # Auto-detect agents on machine
│   │   ├── claude.rs        # Built-in claude adapter
│   │   └── codex.rs         # Built-in codex adapter
│   ├── bundle/              # Bundle management
│   │   └── mod.rs
│   ├── config/              # TOML config loading + resolution
│   │   ├── mod.rs
│   │   └── types.rs         # Config structs
│   └── output/              # Output formatting (text, json, color)
│       └── mod.rs
└── tests/
    └── ...
```

Alternative considered: Cargo workspace with `skittle-core` + `skittle-cli` crates. Rejected — premature for phase 1. Easy to extract later if needed.

### CLI framework: clap with derive macros

Use `clap` with `#[derive(Parser)]` for all command/subcommand definitions. Clap provides:
- Automatic `--help`, `-h`, and `help` subcommand at every level
- Typed argument parsing with validation
- Shell completion generation
- Consistent flag handling

Alternative considered: `argh` (simpler, smaller). Rejected — clap's ecosystem, completion support, and help formatting are worth the dependency.

### Config format: TOML with serde

Single config file at `~/.config/skittle/config.toml`. Serde derives for all config types. The config is the source of truth for sources, targets, adapters, and bundles.

Alternative considered: YAML. Rejected — TOML is the Rust ecosystem standard and better for human editing.

### Source fetching: git2 + reqwest

Use `git2` (libgit2 bindings) for cloning git repos. Use `reqwest` for HTTP fetches. Local file sources use `std::fs` directly.

Alternative considered: shelling out to `git` CLI. Rejected — `git2` gives us programmatic control and avoids PATH dependency issues. However, `git2` has a heavy C dependency (libgit2). If build complexity becomes an issue, fall back to `git` CLI via `std::process::Command`.

### XDG paths: dirs crate

- Config: `~/.config/skittle/config.toml`
- Data (registry/cache): `~/.local/share/skittle/`
  - `sources/` — cached source content
  - `registry.json` — index of all sources, plugins, skills

Alternative considered: Custom path logic. Rejected — `dirs` handles cross-platform XDG correctly.

### Registry format: JSON index + filesystem cache

The registry is a JSON index file (`registry.json`) that maps source:plugin/skill identifiers to cached filesystem paths. Source content is cached as-is in the data directory. This avoids a database while keeping lookups fast.

```
~/.local/share/skittle/
├── registry.json
└── sources/
    ├── trent-skills/            # cached source
    │   ├── openspec/            # plugin dir
    │   │   ├── plugin.toml
    │   │   └── skills/
    │   │       ├── explore/
    │   │       │   └── SKILL.md
    │   │       └── apply/
    │   │           └── SKILL.md
    │   └── ...
    └── ...
```

### Target adapters: trait + TOML-driven

Define an `Adapter` trait:
```rust
trait Adapter {
    fn install_skill(&self, skill: &Skill, target_path: &Path) -> Result<()>;
    fn uninstall_skill(&self, skill_name: &str, target_path: &Path) -> Result<()>;
    fn installed_skills(&self, target_path: &Path) -> Result<Vec<String>>;
}
```

Built-in adapters (`claude`, `codex`) implement this trait directly. Custom adapters are defined in TOML and backed by a generic `TomlAdapter` that reads the config:

```toml
[adapter.my-agent]
skill_dir = "prompts/{name}"
skill_file = "prompt.md"
format = "agentskills"          # passthrough (phase 1 only format)
copy_dirs = ["scripts", "references", "assets"]
```

### Skill identity: source:plugin/skill

Skills are identified as `plugin/skill` by default. Full qualification `source:plugin/skill` required only on collision. The registry maintains a uniqueness index and reports conflicts at `source add` time.

### Bundle swap: clean replace

`bundle swap <from> <to>` uninstalls all skills from bundle `<from>`, then installs all skills from bundle `<to>`. No diff logic — full replacement. The active bundle per target is tracked in the registry.

### Error handling: thiserror + anyhow

Use `thiserror` for library error types, `anyhow` in CLI handlers for ergonomic error propagation with context.

### Output formatting

All commands support `--json` for machine-readable output. Text output uses `colored` crate for terminal colors, respecting `--color auto|always|never` and `NO_COLOR` env var. Verbose mode (`-v`) adds detail. Quiet mode (`-q`) suppresses non-error output.

## Risks / Trade-offs

- **git2 build complexity** → If libgit2 compilation causes issues on some platforms, fall back to shelling out to `git` CLI. Monitor during development.
- **Registry as JSON file** → Could become slow with thousands of skills. Acceptable for phase 1; migrate to SQLite if needed.
- **Single config file** → Could get large with many sources/targets/bundles. Acceptable for phase 1; support `include` directives later.
- **Passthrough-only format** → Phase 1 only supports agents that consume Agent Skills format natively. Cursor/Gemini support requires format converters in phase 2.
- **No lockfile** → Source versions aren't pinned. Two machines with same config may resolve differently. Consider adding `skittle.lock` in phase 2.
