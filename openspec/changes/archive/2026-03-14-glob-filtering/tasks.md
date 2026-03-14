## 1. Dependencies

- [x] 1.1 Add `glob-match` crate to Cargo.toml

## 2. Registry

- [x] 2.1 Add `match_skills(pattern)` method to `Registry` that matches glob pattern against full `source:plugin/skill` identity strings and returns matching triples
- [x] 2.2 Add `is_glob(input)` helper function that returns true if input contains `*`, `?`, or `[`
- [x] 2.3 Add `expand_pattern(input)` helper that prepends `*:` if input has no `:`
- [x] 2.4 Add unit tests for `match_skills`, `is_glob`, and `expand_pattern`

## 3. CLI — skittle list

- [x] 3.1 Change `Command::List` arg from `name: Option<String>` to `patterns: Vec<String>`
- [x] 3.2 Update handler: no args = all skills, each arg checked for glob vs exact, results unioned and deduplicated
- [x] 3.3 Support glob filtering in both default and `--json` output modes
- [x] 3.4 Show informational message when glob matches nothing

## 4. CLI — bundle add

- [x] 4.1 Update `BundleCommand::Add` handler to expand glob patterns via `match_skills`
- [x] 4.2 Store fully qualified `source:plugin/skill` identities in bundle config
- [x] 4.3 Print count of matched and added skills
- [x] 4.4 Error when glob pattern matches no skills

## 5. CLI — bundle list

- [x] 5.1 Change `BundleCommand::List` from no args to `patterns: Vec<String>`
- [x] 5.2 Filter bundle names by glob patterns when provided (union of matches)
- [x] 5.3 Show informational message when filter matches no bundles
