## 1. Source Module Unit Tests

- [ ] 1.1 Add unit tests to `source/detect.rs`: detect() for all four structure types, empty dir error, has_skill_frontmatter() true/false, parse_skill_name() present/missing, parse_skill_description() missing
- [ ] 1.2 Add unit tests to `source/discover.rs`: discover_plugins() multi-plugin, hidden dir skip, empty dir; discover_skills() happy path, missing frontmatter skip, empty plugin
- [ ] 1.3 Add unit tests to `source/fetch.rs`: fetch() local directory, fetch() local single file, fetch() nonexistent path error, copy_dir_recursive() skips .git
- [ ] 1.4 Add unit tests to `source/manifest.rs`: load_source_manifest() with [source] wrapper, flat form, missing name, empty name, file not found, invalid TOML; load_plugin_manifest() happy path, missing name
- [ ] 1.5 Add unit tests to `source/normalize.rs`: normalize() for SingleFile, FlatSkills, SinglePlugin, FullSource variants
- [ ] 1.6 Add unit tests to `source/url.rs`: parse relative path `./`, home expansion `~/`, invalid/empty input error

## 2. Config & Registry Unit Tests

- [ ] 2.1 Add unit tests to `config/mod.rs`: config_path() with override and default, load_from() nonexistent returns default, load_from() invalid TOML returns error, save_to()/load_from() roundtrip
- [ ] 2.2 Add unit tests to `config/types.rs`: adapter format default to "agentskills", invalid TOML deserialization error
- [ ] 2.3 Add unit tests to `registry/mod.rs`: find_plugin() found/not found, load_registry() corrupted JSON error, save_registry()/load_registry() roundtrip

## 3. Output & CLI Helper Unit Tests

- [ ] 3.1 Add unit tests to `output/mod.rs`: Output::from_flags() construction, quiet mode suppresses non-error output, verbose enables debug, non-verbose suppresses debug, json mode emits valid JSON
- [ ] 3.2 Add unit tests for `dir_size()` and `format_size()` in `cli/mod.rs`: dir with files, empty dir, nonexistent path returns 0, format_size byte/KB/MB formatting

## 4. Functional Integration Tests — Source & Target Operations

- [ ] 4.1 Create `tests/functional_source_ops.rs`: add local source and verify registry, add source with custom name, remove source cleans registry, remove source with installed skills fails without force, list sources, show source detail, update source re-detects
- [ ] 4.2 Create `tests/functional_target_ops.rs`: add target with agent/path, default scope/sync values, remove target, add duplicate name fails, list targets

## 5. Functional Integration Tests — Plugin, Skill & Install Operations

- [ ] 5.1 Create `tests/functional_skill_plugin_ops.rs`: list plugins across sources, list plugins filtered by source, show plugin detail, list skills across sources, list skills filtered by plugin, show skill detail, show nonexistent skill error
- [ ] 5.2 Create `tests/functional_install_ops.rs`: install --skill by identity, install --plugin, install --bundle, install --target, install nonexistent skill/plugin fails, uninstall --skill, uninstall --bundle

## 6. Functional Integration Tests — Bundle, Status, Config & Cache

- [ ] 6.1 Create `tests/functional_bundle_ops.rs`: create bundle and add skills, delete bundle, delete active bundle without force fails, drop skill from bundle, swap bundle, create duplicate name fails
- [ ] 6.2 Create `tests/functional_status_config_cache.rs`: status with sources/targets/skills, status with empty config, config show, cache clean removes cached sources

## 7. Error Path & Edge Case Tests

- [ ] 7.1 Add error-path tests across existing unit test modules: empty source name, unknown agent type, install/uninstall with no flags, bundle add with invalid identity
- [ ] 7.2 Add frontmatter edge case tests in `source/detect.rs`: extra whitespace, quoted values, empty frontmatter, incomplete frontmatter (no closing ---)
- [ ] 7.3 Add manifest edge case tests in `source/manifest.rs`: empty plugins list, optional fields present, unknown fields ignored

## 8. Dry-Run Verification Tests

- [ ] 8.1 Add dry-run tests in functional integration files: dry-run install writes nothing, dry-run uninstall removes nothing, dry-run source add modifies nothing, dry-run cache clean removes nothing

## 9. Verify & Stabilize

- [ ] 9.1 Run full `cargo test` suite, fix any failures, ensure all new tests pass
- [ ] 9.2 Verify no production code changes were needed (tests are purely additive)
