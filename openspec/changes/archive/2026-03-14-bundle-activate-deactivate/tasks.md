## 1. Remove active_bundles state

- [x] 1.1 Remove `active_bundles` field from `Registry` in `src/registry/types.rs`
- [x] 1.2 Remove `set_active_bundle`, `active_bundle`, `clear_active_bundle` methods from `src/registry/mod.rs`
- [x] 1.3 Remove active_bundles tests from `src/registry/mod.rs`
- [x] 1.4 Remove active_bundles references from `status` command handler in `src/cli/mod.rs`
- [x] 1.5 Remove active_bundles references from `bundle delete` handler
- [x] 1.6 Remove active_bundles references from `bundle list` handler (drop "ACTIVE ON" column)
- [x] 1.7 Remove active_bundles reference from `apply --bundle` handler (if it sets active bundle)

## 2. Remove bundle swap

- [x] 2.1 Remove `BundleCommand::Swap` variant from enum
- [x] 2.2 Remove swap handler from `src/cli/mod.rs`

## 3. Add activate command

- [x] 3.1 Add `BundleCommand::Activate` variant: `{ name: String, target: Option<String>, all: bool, force: bool }`
- [x] 3.2 Implement activate handler: validate bundle exists, validate target (or iterate all), install each skill idempotently, respect `--force` / dry-run
- [x] 3.3 Skip silently when a skill is already installed on the target

## 4. Add deactivate command

- [x] 4.1 Add `BundleCommand::Deactivate` variant: `{ name: String, target: Option<String>, all: bool, force: bool }`
- [x] 4.2 Implement deactivate handler: validate bundle exists, validate target (or iterate all), uninstall each skill idempotently, respect `--force` / dry-run
- [x] 4.3 Skip silently when a skill is not installed on the target

## 5. Update bundle list

- [x] 5.1 Simplify `bundle list` to show only NAME and SKILLS columns (remove ACTIVE ON)

## 6. Tests

- [x] 6.1 CLI flag test: `bundle activate` parses with target, with --all, and errors on both/neither
- [x] 6.2 CLI flag test: `bundle deactivate` parses with target, with --all
- [x] 6.3 CLI flag test: `bundle swap` is no longer a valid subcommand
- [x] 6.4 Update shell test suites that reference swap or active bundles
