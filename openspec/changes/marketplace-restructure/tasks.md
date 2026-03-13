## 1. Directory restructure

- [x] 1.1 Rename `cache_dir()` from `data_dir()/sources` to `data_dir()/external`
- [x] 1.2 Create `plugins_dir()` function returning `data_dir()/plugins`
- [x] 1.3 Create `skittle_internal_dir()` function returning `data_dir()/.skittle`
- [x] 1.4 Move registry load/save to `.skittle/registry.json`
- [x] 1.5 Update `skittle init` to create `plugins/`, `external/`, `.skittle/` directories
- [x] 1.6 Update `skittle init` to generate `.gitignore` (ignores `external/`, `.skittle/`)
- [x] 1.7 Add legacy migration: rename `sources/` to `external/` if present during init
- [x] 1.8 Update default config template to reflect new layout
- [x] 1.9 Update all tests referencing `sources/` path to use `external/`

## 2. Provenance tracking

- [x] 2.1 Add `installed` field to Registry struct: `HashMap<String, HashMap<String, InstalledSkill>>`
- [x] 2.2 Define `InstalledSkill` struct: source, plugin, skill, origin (relative path)
- [x] 2.3 Record provenance in registry during install (compute origin path from source/plugin/skill)
- [x] 2.4 Add unit tests for provenance recording and lookup

## 3. Collect command

- [x] 3.1 Add `Command::Collect` variant with `--skill`, `--target`, `--adopt`, `--force` flags
- [x] 3.2 Implement collect for tracked skill: look up provenance, copy target → origin
- [x] 3.3 Implement collect with `--adopt`: copy to `plugins/<plugin>/skills/<skill>/`, create plugin.json if needed
- [x] 3.4 Implement collect without `--skill` (scan target): list tracked vs untracked, prompt to adopt
- [x] 3.5 Implement `--force` on scan: adopt all untracked without prompting
- [x] 3.6 Add unit tests for collect: tracked skill, adopt, untracked detection

## 4. Marketplace generation

- [x] 4.1 Add `generate_marketplace()` function: scan `plugins/`, produce marketplace.json
- [x] 4.2 Write marketplace.json to `.claude-plugin/marketplace.json` in data dir
- [x] 4.3 Call `generate_marketplace()` after collect `--adopt` and after init (if plugins/ has content)
- [x] 4.4 Add unit tests for marketplace generation: multiple plugins, empty plugins, plugins without plugin.json

## 5. Source pinning

- [x] 5.1 Add optional `ref` field to `SourceConfig` in config types
- [x] 5.2 Add `--ref` flag to `skittle add` command
- [x] 5.3 Update `fetch_git()` to accept optional ref: use `--branch <ref>` for tags/branches, checkout SHA for commits
- [x] 5.4 Update `update_git()` to checkout pinned ref after fetch
- [x] 5.5 Add unit tests for ref pinning: tag, branch, commit SHA, no ref

## 6. Test fixture and harness updates

- [x] 6.1 Update test fixtures to use `external/` paths
- [x] 6.2 Update harness setup.sh cache path references
- [x] 6.3 Update harness suite tests referencing `sources/` to `external/`
- [x] 6.4 Add harness tests for collect command
- [x] 6.5 Add harness test for marketplace.json generation
- [x] 6.6 Run full test harness and verify all pass

## 7. Spec sync

- [x] 7.1 Sync delta specs to main specs
