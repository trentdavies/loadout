## Context

`skittle init` creates dirs + default config. The new flow adds three optional steps after directory creation: git init, target detection, and marketplace sourcing. Each step prompts once with a sensible default.

## Goals / Non-Goals

**Goals:**
- One-command setup from zero to working skittle
- Git-versioned data dir by default
- Auto-detect and register agent targets
- Offer popular marketplaces as sources
- Maintainable marketplace list

**Non-Goals:**
- Interactive skill installation during init (that's `apply`)
- Marketplace discovery/search (just a curated list)
- Git remote setup (user does that themselves if they want)

## Decisions

### 1. `git init` at data dir root

Run `git init` in the skittle data dir (`~/.local/share/skittle/`). This puts `plugins/`, `config.toml` (via symlink or copy), and any local skill files under version control. The existing `.gitignore` already excludes `external/` (cached sources) and `.skittle/` (registry).

Skip silently if `.git` already exists. Skip silently if `git` is not installed (warn in verbose mode).

### 2. Target detection reuses existing logic

Extract the scanning logic from `TargetCommand::Detect` into a shared function `detect_targets(config, home) -> Vec<(agent, path, already_registered)>`. Call it from both `target detect` and `init`. During init, auto-add all unregistered targets (no per-target prompt) — the user already opted in.

### 3. Marketplace list as a const

Store the list as a `const` array of `(&str, &str)` tuples (name, GitHub URL) in a dedicated `src/marketplace.rs` module. This is simpler than an external config file and easy to update. Example entries:

```rust
pub const KNOWN_MARKETPLACES: &[(&str, &str)] = &[
    ("Anthropic Skills", "https://github.com/anthropics/skills.git"),
    ("Anthropic Plugins", "https://github.com/anthropics/claude-plugins-official.git"),
];
```

### 4. Multi-select prompt for marketplaces

Use `dialoguer::MultiSelect` to let the user pick from the list. Default: all selected. In non-interactive mode: skip (don't add any marketplaces without explicit user choice).

### 5. Init flow sequence

```
skittle init [url]
  │
  ├─ Create dirs + config (existing)
  │
  ├─ git init data dir? [Y/n]
  │   └─ git init if yes
  │
  ├─ URL provided?
  │   ├─ Yes → add source (existing behavior), skip marketplace prompt
  │   └─ No → continue to marketplace prompt
  │
  ├─ Detect agent targets? [Y/n]
  │   └─ scan + auto-add all found
  │
  └─ Add popular skill sources? (multi-select)
      └─ fetch + register each selected marketplace
```

### 6. Quiet/non-interactive defaults

- git init: yes (always, it's non-destructive)
- target detect: yes (auto-add all found)
- marketplaces: skip (requires explicit selection)

## Risks / Trade-offs

- **Hardcoded marketplace URLs**: If a repo is renamed/deleted, init will fail for that entry. Mitigation: each marketplace add is wrapped in a try, failures are warnings not errors.
- **git init may surprise users**: Some users may not want their data dir to be a git repo. Mitigation: the prompt defaults to yes but can be declined; `--quiet` still does it since it's non-destructive.
