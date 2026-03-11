# AGENTS.md — Skittle Project

## Critical Safety Rules

**NO DESTRUCTIVE ACTIONS ON THE HOST SYSTEM.** This means:
- **NEVER** run `skittle install`, `skittle uninstall`, `skittle source remove`, `skittle target remove`, or any command that modifies skill installations on this laptop
- **NEVER** run `skittle` commands against real agent targets (~/.claude, ~/.codex, ~/.cursor, etc.)
- **NEVER** modify, delete, or overwrite any installed skills on the host machine
- The only safe `skittle` commands to run locally are `cargo check`, `cargo build`, and `cargo test`

## Testing

**ALL testing MUST use the Docker test harness.** No exceptions.

```bash
# Build and run the full test suite
docker build -t skittle-test -f tests/Dockerfile . && docker run --rm skittle-test

# Run a specific suite
docker run --rm skittle-test /tests/harness/runner.sh --suite 02
```

The Docker container provides:
- Isolated XDG paths (`/tmp/test-config`, `/tmp/test-data`)
- Mock target directories (`/tmp/test-targets/claude`, `/tmp/test-targets/codex`)
- Test fixtures at `/tests/fixtures/`
- Network tests disabled by default (`SKIP_NETWORK=1`)

**Do NOT test skittle commands directly on the host.** Use `cargo check` to verify compilation, then Docker for functional tests.

**Do NOT modify test harness files** (`tests/harness/`, `tests/fixtures/`, `tests/Dockerfile`) as part of implementing CLI tasks. The test harness is the spec — fix the CLI to pass the tests, never the other way around.

## Project Structure

- `src/` — Rust CLI source (clap derive, TOML config, XDG paths)
- `tests/harness/` — Bash test harness (runner, lib, setup, 12 suite files)
- `tests/fixtures/` — Test fixture data (skills, plugins, sources)
- `tests/Dockerfile` — Docker container for isolated testing
- `openspec/` — Change specifications and task tracking

## Implementation Workflow

Red-green testing against `openspec/changes/skittle-cli-v1/tasks.md`:
1. Pick the next unchecked task
2. Implement the code changes
3. Run `cargo check` to verify compilation
4. Run Docker test harness: `docker build -t skittle-test -f tests/Dockerfile . && docker run --rm skittle-test`
5. Verify pass count has NOT regressed (baseline: 168 passed as of task 5.2). New tests SHOULD turn green as features land.
6. If tests regressed, fix before proceeding — do NOT commit broken code
7. Mark the task `[x]` in tasks.md
8. Commit with a descriptive message (include pass/fail count in commit body)

## Key Conventions

- Config: TOML at `$XDG_CONFIG_HOME/skittle/config.toml`
- Data: JSON registry + cache at `$XDG_DATA_HOME/skittle/`
- CLI: clap derive, global flags (-n, -v, -q, --json, --color, --config)
- All commands must support --help, -h, help
- Exit 0 on success, non-zero on error
- `install`/`uninstall` require explicit flags (--all, --skill, --plugin, --bundle)
