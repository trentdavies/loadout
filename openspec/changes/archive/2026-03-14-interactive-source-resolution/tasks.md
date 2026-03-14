## 1. Prompt module

- [x] 1.1 Add `dialoguer` dependency to Cargo.toml
- [x] 1.2 Create `src/prompt/mod.rs` with `is_interactive()` (stdin isatty check)
- [x] 1.3 Implement `confirm_or_override(label, default, quiet) -> String` using dialoguer Input with default value; return default when non-interactive or quiet
- [x] 1.4 Implement `select_from(label, options, quiet) -> Result<String>` using dialoguer Select; error when non-interactive or quiet
- [x] 1.5 Register `mod prompt` in `src/lib.rs`

## 2. Add command changes

- [x] 2.1 Rename `--name` to `--source` in `Command::Add` struct; add `--plugin` and `--skill` optional flags
- [x] 2.2 Add backward-compat error: if `--name` is passed (via clap hidden alias or manual check), exit with message "`--name` has been renamed to `--source`"
- [x] 2.3 After URL parse + `default_name()`, prompt for source name confirmation when `--source` not provided and interactive
- [x] 2.4 After `fetch` + `detect`, determine inferred plugin/skill names from structure type
- [x] 2.5 Prompt for plugin name when `--plugin` not provided, interactive, and plugin name differs from source name
- [x] 2.6 Prompt for skill name when `--skill` not provided, interactive, and source is single-skill (SingleFile/SingleSkillDir)
- [x] 2.7 Pass overridden names into `normalize` (extend normalize to accept optional name overrides)
- [x] 2.8 In non-interactive/quiet mode, print resolved identities to stderr (unless `--quiet`)

## 3. Remove command changes

- [x] 3.1 Make the `name` positional argument optional in `Command::Remove` struct
- [x] 3.2 When name is omitted and interactive: list registered sources via `select_from`, then proceed with selected name
- [x] 3.3 When name is omitted and non-interactive: exit with error "source name required"

## 4. Normalize overrides

- [x] 4.1 Extend `normalize()` signature to accept `Option<&str>` overrides for plugin and skill names
- [x] 4.2 Apply overrides in each structure branch (SingleFile, SinglePlugin, FlatSkills, Marketplace, SingleSkillDir)
- [x] 4.3 Validate overridden names are kebab-case; error if not

## 5. Tests

- [x] 5.1 Unit test `is_interactive()` returns false when stdin is not a TTY (default in test harness)
- [x] 5.2 Unit test `confirm_or_override` returns default in non-interactive mode
- [x] 5.3 Unit test `select_from` returns error in non-interactive mode
- [x] 5.4 Integration test: `skittle add <url> --source s --plugin p --skill sk` bypasses prompts and uses provided names
- [x] 5.5 Integration test: `skittle add <url> --quiet` uses inferred defaults without prompting
- [x] 5.6 Integration test: `skittle remove` without name in non-TTY exits with error
- [x] 5.7 Integration test: `skittle add <url> --name foo` exits with deprecation error
