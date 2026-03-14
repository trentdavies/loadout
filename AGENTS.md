# AGENTS.md — Loadout Project

## Critical Safety Rules

**NO DESTRUCTIVE ACTIONS ON THE HOST SYSTEM.** This means:
- **NEVER** run `loadout install`, `loadout uninstall`, `loadout source remove`, `loadout target remove`, or any command that modifies skill installations on this laptop
- **NEVER** run `loadout` commands against real agent targets (~/.claude, ~/.codex, ~/.cursor, etc.)
- **NEVER** modify, delete, or overwrite any installed skills on the host machine
- The only safe `loadout` commands to run locally are `cargo check`, `cargo build`, and `cargo test`

## Testing

**ALL testing MUST use the Docker test harness.** No exceptions.

```bash
# Build and run the full test suite
docker build -t loadout-test -f tests/Dockerfile . && docker run --rm loadout-test

# Run a specific suite
docker run --rm loadout-test /tests/harness/runner.sh --suite 02
```

The Docker container provides:
- Isolated XDG paths (`/tmp/test-config`, `/tmp/test-data`)
- Mock target directories (`/tmp/test-targets/claude`, `/tmp/test-targets/codex`)
- Test fixtures at `/tests/fixtures/`
- Network tests disabled by default (`SKIP_NETWORK=1`)

**Do NOT test loadout commands directly on the host.** Use `cargo check` to verify compilation, then Docker for functional tests.

**Do NOT modify test harness files** (`tests/harness/`, `tests/fixtures/`, `tests/Dockerfile`) as part of implementing CLI tasks. The test harness is the spec — fix the CLI to pass the tests, never the other way around.

## Project Structure

- `src/` — Rust CLI source (clap derive, TOML config, XDG paths)
- `tests/harness/` — Bash test harness (runner, lib, setup, 12 suite files)
- `tests/fixtures/` — Test fixture data (skills, plugins, sources)
- `tests/Dockerfile` — Docker container for isolated testing
- `openspec/` — Change specifications and task tracking

## Implementation Workflow

Red-green testing against `openspec/changes/loadout-cli-v1/tasks.md`:
1. Pick the next 2-3 unchecked tasks and implement them in sequence
2. For each task: implement, run `cargo check`, mark `[x]`, commit
3. After every 5 completed tasks, run Docker test harness:
   `docker build -t loadout-test -f tests/Dockerfile . && docker run --rm loadout-test`
4. Verify pass count has NOT regressed (baseline: 190 passed as of task 5.4). New tests SHOULD turn green as features land.
5. If tests regressed, fix before proceeding — do NOT commit broken code
6. Include pass/fail count in commit body when Docker tests are run

## Key Conventions

- Config: TOML at `$XDG_CONFIG_HOME/loadout/loadout.toml`
- Data: JSON registry + cache at `$XDG_DATA_HOME/loadout/`
- CLI: clap derive, global flags (-n, -v, -q, --json, --color, --config)
- All commands must support --help, -h, help
- Exit 0 on success, non-zero on error
- `install`/`uninstall` require explicit flags (--all, --skill, --plugin, --bundle)
