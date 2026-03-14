## 1. Marketplace list

- [x] 1.1 Create `src/marketplace.rs` with `KNOWN_MARKETPLACES: &[(&str, &str)]` const array
- [x] 1.2 Populate with initial entries (Anthropic Skills, Anthropic Plugins, etc.)
- [x] 1.3 Register `mod marketplace` in `src/lib.rs`

## 2. Multi-select prompt

- [x] 2.1 Add `multi_select(label, options, defaults, quiet) -> Vec<usize>` to `src/prompt.rs` using `dialoguer::MultiSelect`; returns empty vec in non-interactive/quiet mode

## 3. Extract target detection

- [x] 3.1 Extract scanning + candidate logic from `TargetCommand::Detect` handler into a shared function `detect_agent_targets() -> Vec<(String, PathBuf)>` (agent, path)
- [x] 3.2 Update `TargetCommand::Detect` to call the shared function
- [x] 3.3 Create `add_detected_targets(config, quiet)` helper that registers all candidates into config (auto-add, no per-target prompt)

## 4. Init wizard

- [x] 4.1 After directory creation: prompt "Initialize git in skittle data dir? [Y/n]" and run `git init` if accepted. Skip if `.git` exists or `git` not on PATH.
- [x] 4.2 Prompt "Detect and add agent targets? [Y/n]" and call `add_detected_targets` if accepted
- [x] 4.3 If no URL argument: present marketplace multi-select using `KNOWN_MARKETPLACES`, fetch and register each selected source (wrap each in try, warn on failure)
- [x] 4.4 In non-interactive/quiet mode: git init (yes), detect targets (yes), skip marketplaces

## 5. Tests

- [x] 5.1 Unit test: `KNOWN_MARKETPLACES` is non-empty and all entries have non-empty name and URL
- [x] 5.2 Unit test: `multi_select` returns empty vec in non-interactive mode
- [x] 5.3 Unit test: `detect_agent_targets` returns without error
- [x] 5.4 Integration test: init creates `.git` directory in data dir
- [x] 5.5 CLI flag test: `init` parses with and without URL argument
