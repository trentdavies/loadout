## 1. Rename install to apply (CLI surface)

- [x] 1.1 Rename `Command::Install` to `Command::Apply` in `src/cli/mod.rs` enum, update help text from "Install skills to targets" to "Apply skills to targets"
- [x] 1.2 Add `--force` / `-f` and `--interactive` / `-i` flags to the `Apply` command variant
- [x] 1.3 Update the match arm in `run()` from `Command::Install` to `Command::Apply`, threading new flags through
- [x] 1.4 Update `cli-framework` spec reference from `install` to `apply` in top-level command list

## 2. Skill comparison in Adapter

- [x] 2.1 Add `SkillStatus` enum (New, Unchanged, Changed) to `src/target/adapter.rs`
- [x] 2.2 Implement `compare_skill()` method on `Adapter` that returns `SkillStatus` by comparing source skill directory against target, byte-for-byte per file
- [x] 2.3 Handle edge cases: files only in source, files only in target, subdirectory comparison in copy_dirs

## 3. Apply logic with overwrite protection

- [x] 3.1 Refactor install loop to call `compare_skill()` before each install, collecting results into (skill, status) pairs
- [x] 3.2 Implement default mode: if any skill is CHANGED, refuse to proceed and print conflicting skill names with suggestion to use `--force` or `-i`
- [x] 3.3 Implement `--force` mode: overwrite all CHANGED skills without prompting
- [x] 3.4 Skip UNCHANGED skills silently in all modes
- [x] 3.5 Apply NEW skills without prompting in all modes

## 4. Interactive mode

- [x] 4.1 Implement interactive prompt loop: display skill name + status, read single-char input from stdin
- [x] 4.2 Handle `s` (skip), `o` (overwrite), `f` (force-all remaining), `q` (quit)
- [x] 4.3 Handle `d` (diff): generate unified diff for all files in the skill directory using `similar` crate, display with `=== <filename> ===` headers and `--- installed` / `+++ source` labels
- [x] 4.4 After showing diff, re-prompt with `[s]kip  [o]verwrite  [q]uit` (no diff option on second prompt)

## 5. Summary output

- [x] 5.1 Track counts: new_applied, updated, unchanged, conflict_skipped
- [x] 5.2 Display summary line: "Applied N skills (X new, Y updated), skipped Z unchanged." with optional ", W conflict skipped." suffix

## 6. Dependencies and cleanup

- [x] 6.1 Add `similar` crate to `Cargo.toml`
- [x] 6.2 Update all test references from `install` to `apply` in test files
- [x] 6.3 Update `AGENTS.md` if it references the install command
