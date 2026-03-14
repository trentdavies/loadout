## Context

Skittle currently stores everything under `~/.local/share/skittle/sources/`. External git clones and local copies live side by side. The registry (`registry.json`) sits at the data root. There's no concept of owned vs external content, no way to push skill modifications back upstream, and the directory isn't independently useful as a Claude marketplace.

The user has also flattened the CLI — `add`, `remove`, `update`, `list` are now top-level commands (not under `source`).

## Goals / Non-Goals

**Goals:**
- Separate `plugins/` (yours, git tracked) from `external/` (theirs, gitignored)
- Generate `.claude-plugin/marketplace.json` from `plugins/` only
- Add `skittle collect` to bring modified skills back from targets
- Track provenance in the registry so collect knows where skills came from
- Add `ref` pinning for git sources in `skittle.toml`
- Move registry to `.skittle/registry.json` (gitignored internals)

**Non-Goals:**
- Managing git workflows inside external sources (user does that themselves)
- Automatic conflict resolution when collecting modified skills
- Plugin dependency management between marketplace plugins
- Syncing marketplace.json to a remote registry

## Decisions

### 1. Directory layout

```
~/.local/share/skittle/
├── .claude-plugin/marketplace.json   ← generated, git tracked
├── .skittle/registry.json            ← internals, gitignored
├── .gitignore                        ← ignores external/, .skittle/
├── skittle.toml                      ← config, git tracked
├── plugins/                          ← yours, git tracked
│   └── <plugin-name>/
│       ├── .claude-plugin/plugin.json
│       └── skills/<skill-name>/SKILL.md
└── external/                         ← cached clones, gitignored
    └── <source-name>/
        └── ...
```

`config::cache_dir()` changes from `data_dir()/sources` to `data_dir()/external`. Registry path changes from `data_dir()/registry.json` to `data_dir()/.skittle/registry.json`.

### 2. Marketplace generation

A function `generate_marketplace()` scans `plugins/` for subdirectories containing `.claude-plugin/plugin.json` or `skills/` subdirectories. Produces a `marketplace.json` at `.claude-plugin/marketplace.json` with entries pointing to `./plugins/<name>`.

Called after: `collect --adopt`, `skittle init` (if plugins/ has content), and any operation that mutates `plugins/`.

NOT called after: `add`, `update`, `install` (these only touch external/ and targets).

### 3. Provenance tracking in registry

The registry gains an `installed` map:

```json
{
  "sources": [...],
  "active_bundles": {...},
  "installed": {
    "claude": {
      "contract-review": {
        "source": "anthropic-plugins",
        "plugin": "legal",
        "skill": "contract-review",
        "origin": "external/anthropic-plugins/legal/skills/contract-review"
      }
    }
  }
}
```

`origin` is the path relative to the skittle data dir. Used by `collect` to copy back.

Written during `install`. Read during `collect`.

### 4. `skittle collect` command

```
skittle collect --skill <name> --target <target>
  → looks up origin in registry
  → copies target skill dir → origin path
  → if origin is external: user can git commit/push from there
  → if origin is plugins/: already tracked

skittle collect --skill <name> --target <target> --adopt
  → copies to plugins/<plugin>/skills/<skill>/
  → creates plugin.json if needed
  → regenerates marketplace.json

skittle collect --target <target>
  → scans target for all skills
  → shows tracked vs untracked
  → prompts to adopt untracked skills
```

### 5. `ref` pinning

```toml
[[source]]
name = "anthropic-plugins"
url = "git@github.com:anthropics/knowledge-work-plugins.git"
type = "git"
ref = "v1.2.0"
```

`ref` is optional, defaults to HEAD. Used during `add` (clone at ref) and `update` (fetch + checkout ref). Supported values: tag, branch name, commit SHA.

Implementation: `git clone --branch <ref>` for tags/branches, `git clone` + `git checkout <sha>` for commits.

### 6. Migration from `sources/` to `external/`

`skittle init` checks for legacy `sources/` directory. If found, renames to `external/`. No data loss, just a rename.

## Risks / Trade-offs

**[Marketplace.json drift]** → Generated file could get out of sync if user manually edits plugins/. Mitigation: regenerate on every `collect --adopt` and provide `skittle marketplace refresh` (or just regenerate on `status`).

**[Provenance lost for pre-migration installs]** → Existing installed skills have no provenance in registry. `collect` will report them as "unknown origin." Mitigation: user can `--adopt` them to claim ownership, or reinstall to establish provenance.

**[Large plugins/ directory]** → If user adopts many skills, plugins/ grows. This is expected and desired — it's their curated marketplace.
