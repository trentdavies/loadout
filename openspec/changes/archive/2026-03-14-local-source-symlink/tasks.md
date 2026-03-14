## 1. Config changes

- [x] 1.1 Add optional `mode` field to `SourceConfig` in `src/config/types.rs` (values: `"symlink"` or `"copy"`, default omitted for backward compat)

## 2. Fetch changes

- [x] 2.1 Add `fetch_local_symlink` function to `src/source/fetch.rs` that creates a directory symlink from cache_path to source_path
- [x] 2.2 Fallback: if symlink creation fails (cross-device), fall back to copy with a warning to stderr
- [x] 2.3 Add `fetch_local_with_mode` (or extend `fetch`) that dispatches to symlink or copy based on mode; SingleFile sources always use copy regardless of mode

## 3. Prompt changes

- [x] 3.1 Add `prompt_fetch_mode(quiet) -> String` to `src/prompt.rs` — returns `"symlink"` or `"copy"`, defaults to symlink in non-interactive/quiet

## 4. CLI add command

- [x] 4.1 Add `--symlink` and `--copy` boolean flags to `Command::Add` (mutually exclusive via clap `conflicts_with`)
- [x] 4.2 After source name confirmation and before fetch: if source is `SourceUrl::Local` and `source_path.is_dir()`, determine fetch mode from flags or prompt; if source is a file, always use copy
- [x] 4.3 Pass fetch mode to `fetch`
- [x] 4.4 Persist `mode` in `SourceConfig` when saving (only for directory sources that used symlink)

## 5. CLI update command

- [x] 5.1 When updating a source with `mode: "symlink"`, skip re-fetch; re-run detect + normalize only
- [x] 5.2 Print "(symlinked, re-detecting)" instead of "fetching" for symlinked sources

## 6. Tests

- [x] 6.1 Unit test: `fetch_local_symlink` creates a symlink that resolves to the original directory
- [x] 6.2 Unit test: `prompt_fetch_mode` returns `"symlink"` in non-interactive mode
- [x] 6.3 Integration test: add local directory source with `--symlink` creates symlink in cache
- [x] 6.4 Integration test: add local directory source with `--copy` creates real copy in cache
- [x] 6.5 Integration test: add local single-file source always copies regardless of `--symlink` flag
- [x] 6.6 Integration test: update symlinked source re-detects without re-fetching
- [x] 6.7 CLI flag test: `--symlink` and `--copy` parse correctly, conflict when both passed
