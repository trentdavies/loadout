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
4. Mark the task `[x]` in tasks.md
5. Commit with a descriptive message
6. Run Docker test harness to verify progress (periodically, not every commit)

## Key Conventions

- Config: TOML at `$XDG_CONFIG_HOME/skittle/config.toml`
- Data: JSON registry + cache at `$XDG_DATA_HOME/skittle/`
- CLI: clap derive, global flags (-n, -v, -q, --json, --color, --config)
- All commands must support --help, -h, help
- Exit 0 on success, non-zero on error
- `install`/`uninstall` require explicit flags (--all, --skill, --plugin, --bundle)
