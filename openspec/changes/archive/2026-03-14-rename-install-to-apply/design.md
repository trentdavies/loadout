## Context

The `install` command in `src/cli/mod.rs` currently calls `adapter.install_skill()` which does blind `fs::copy` operations. There is no comparison between source and target content. The `Adapter` struct in `src/target/adapter.rs` handles file operations but has no concept of "already exists and differs."

The `collect` command goes the other direction (target → source) and can safely always overwrite because the source directory is git-managed.

## Goals / Non-Goals

**Goals:**
- Rename `install` → `apply` across CLI, code, and specs
- Default behavior: refuse to overwrite skills that differ, exit with actionable error
- `--force` / `-f`: overwrite all without prompting
- `--interactive` / `-i`: per-skill resolution with skip/overwrite/diff/force-all/quit
- Byte-level comparison at apply time — no stored hashes
- Clean summary output: "Applied N skills (X new, Y updated), skipped Z unchanged."

**Non-Goals:**
- 3-way merge or content hashing in the registry
- Per-file conflict granularity (per-skill only)
- Changes to `collect` behavior
- Changes to `uninstall` behavior (will be renamed to `remove` in separate change if desired)

## Decisions

### 1. Comparison strategy: byte-level directory comparison, no hashing

Compare every file in the source skill directory against the corresponding file at the target. If any file differs (content or existence), the skill is flagged as CHANGED.

**Why over hashing**: Simpler, no registry schema changes, no stale hash bugs. The comparison only runs at apply time when we already have both directories available. Cost is negligible for the file sizes involved (SKILL.md + a few scripts).

**States**:
- NEW: skill directory doesn't exist at target
- UNCHANGED: all files match byte-for-byte
- CHANGED: at least one file differs (content, missing in target, or extra in source)

### 2. Comparison lives in the Adapter, not in CLI logic

Add a `compare_skill()` method to `Adapter` that returns a `SkillStatus` enum (New/Unchanged/Changed). The CLI orchestrates the flow but delegates comparison to the adapter.

**Why**: The adapter already knows the skill directory layout and file mapping. Keeps the CLI focused on user interaction.

### 3. Interactive mode uses stdin line reader, not a TUI library

Simple `stdin().read_line()` prompts. No dependency on `dialoguer`, `inquire`, or similar crates.

**Why**: Skittle has minimal dependencies. A line reader is sufficient for single-character choices. Adding a TUI library for 5 prompts is overkill.

### 4. Diff generation uses the `similar` crate

For displaying unified diffs in interactive mode, use the `similar` crate which provides unified diff output with no external process dependencies.

**Why over shelling out to `diff`**: Cross-platform, no PATH dependency, clean API. `similar` is a well-maintained Rust crate with minimal transitive dependencies.

### 5. Force-all in interactive mode applies to remaining skills only

When user selects `[f]orce-all`, all remaining skills in the current apply run are overwritten without further prompting. Already-skipped skills are not revisited.

**Why**: Forward-only is the least surprising behavior. Revisiting skipped skills would require tracking state and re-prompting.

## Risks / Trade-offs

- **[Breaking change]** → Scripts using `skittle install` will break. Mitigation: clear error message if someone tries `install` suggesting `apply`.
- **[No 3-way merge]** → Can't distinguish "source changed" from "target edited." Mitigation: Acceptable given `collect` exists for the reverse direction and the per-skill granularity keeps decisions manageable.
- **[New dependency]** → `similar` crate added for diff display. Mitigation: It's a pure-Rust crate with no transitive deps, widely used.
- **[stdin blocking]** → Interactive mode blocks on user input. Mitigation: This is expected CLI behavior. Non-interactive use has `--force` and default-refuse modes.
