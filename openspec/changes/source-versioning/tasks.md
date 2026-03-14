## 1. Fix clone to honor ref

- [x] 1.1 Change `fetch_git` signature to accept `Option<&str>` for the ref
- [x] 1.2 When ref is provided, use `git clone --branch <ref> --depth 1` instead of plain `--depth 1`
- [x] 1.3 Remove the post-clone `git checkout` workaround from the `add` command in `cli/mod.rs`
- [x] 1.4 Pass the ref from `SourceConfig` through to `fetch_git` in both `add` and `update` paths

## 2. Ref type detection

- [x] 2.1 Add `is_tag(ref, repo_path) -> bool` function in `src/source/fetch.rs` that runs `git tag --list <ref>` and returns true if output is non-empty
- [x] 2.2 Add `RefType` enum (Latest, Tracking, Pinned) and `detect_ref_type(ref, repo_path)` function

## 3. Fix update to respect ref

- [x] 3.1 Modify `update_git` to accept `Option<&str>` for the ref
- [x] 3.2 When ref is a tag (pinned): warn and return early without fetching
- [x] 3.3 When ref is a branch (tracking): fetch and `git reset --hard origin/<branch>`
- [x] 3.4 When no ref (latest): keep current behavior (`origin/HEAD`)
- [x] 3.5 Thread the stored ref from config into the `update` command's call to `update_git`

## 4. Ref switching via update --ref

- [x] 4.1 Add `--ref` flag to the `Update` command variant in `cli/mod.rs`
- [x] 4.2 When `--ref` is provided: fetch, checkout new ref, update config, re-detect and re-register skills
- [x] 4.3 Handle `--ref latest` as a special case that removes the ref from config

## 5. Display ref in output

- [x] 5.1 Show ref in `skittle list` output (append to source identity or add a column)
- [x] 5.2 Show ref per source in `skittle status` output
- [x] 5.3 Include ref in `skittle list --json` output

## 6. Tests

- [x] 6.1 Update existing fetch tests for the new `fetch_git` signature
- [x] 6.2 Add unit test for `is_tag` detection
- [x] 6.3 Update sandbox test suite to exercise `--ref` on add and update
