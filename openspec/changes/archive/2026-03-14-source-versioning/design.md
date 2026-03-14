## Context

`fetch_git` in `src/source/fetch.rs` currently does `git clone --depth 1 <url>` ignoring any ref. After clone, the `add` command does a bare `git checkout <ref>` which fails silently on a shallow clone that doesn't have the ref's objects. `update_git` does `git fetch origin` + `git reset --hard origin/HEAD`, always landing on the default branch regardless of config.

`SourceConfig` already has `ref: Option<String>` and the CLI already accepts `--ref` on `add`. The config storage is fine — the fetch/update logic just doesn't use it.

## Goals / Non-Goals

**Goals:**
- `git clone --branch <ref> --depth 1` when ref is provided (works for tags and branches)
- `update` pulls latest for branches, warns+skips for tags
- `update --ref <new-ref>` switches version in place (fetch + checkout + update config)
- Detect tag vs branch via `git tag --list <ref>`
- Show ref in `list` and `status` output

**Non-Goals:**
- Commit SHA pinning (not supported by `--branch` with `--depth 1`)
- Lockfiles or hash verification
- Automatic "new version available" notifications

## Decisions

### 1. Clone strategy: `--branch <ref> --depth 1`

`git clone --branch <ref> --depth 1` works for both tags and branches and keeps clones fast. No need for full clones.

**Why not full clone**: The repos we're cloning are skill repositories, often large (anthropic/skills has hundreds of files). Shallow clones save significant time and disk.

### 2. Tag vs branch detection: `git tag --list`

After clone or on update, run `git tag --list <ref>` in the repo. If it returns output, it's a tag (pinned). Otherwise treat it as a branch (tracking).

**Why not `git show-ref`**: `git tag --list` is simpler and works correctly on shallow clones. `show-ref` can be ambiguous when a name exists as both tag and branch.

### 3. Update behavior by ref type

| Ref Type | Update Action |
|----------|--------------|
| None (latest) | `git fetch origin` + `git reset --hard origin/HEAD` (current behavior) |
| Branch | `git fetch origin` + `git reset --hard origin/<branch>` |
| Tag | Warn "source 'x' is pinned to <tag>, skipping" and return |

### 4. Ref switching: `update --ref`

`skittle update <name> --ref <new-ref>` does:
1. `git fetch origin`
2. `git checkout <new-ref>` (may need `origin/<ref>` for branches)
3. Update `SourceConfig.ref` in config file
4. Re-detect and re-register skills

This reuses the existing update flow but adds the ref switch before re-detection.

### 5. Pass ref through fetch API

Change `fetch_git` signature to accept `Option<&str>` for the ref. The `add` command passes it from CLI args. The `update` command passes it from stored config. Removes the post-clone `git checkout` hack from the `add` command.

## Risks / Trade-offs

- **[Shallow clone limitation]** → Can't pin to arbitrary commit SHAs with `--depth 1 --branch`. Mitigation: Document that `--ref` supports tags and branches only. SHA pinning is a rare need for skill repos.
- **[Tag/branch ambiguity]** → A ref name could theoretically exist as both. Mitigation: `git clone --branch` prefers tags, which is the right default for versioning. Document behavior.
- **[Network on ref switch]** → Switching refs requires a fetch. Mitigation: This is expected — you're changing versions.
