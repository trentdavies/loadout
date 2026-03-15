# Loadout — Agent Skill Manager

Loadout is a Rust CLI that sources, caches, and installs agent skills across coding agents (Claude, Codex, Cursor). It treats skill management as a package management problem: sources provide skills, agents consume them, a registry tracks provenance.

## Build & Test Commands

```bash
just build           # debug build
just release         # release build
just test            # rust tests (unit + integration)
just harness         # bash test harness in Docker (offline, no network)
just sandbox-test    # sandbox tests in Docker (network-dependent, real git repos)
just test-all        # rust + harness combined
just check           # cargo fmt --check + clippy -D warnings
just fix             # auto-fix fmt + clippy
```

All Rust tests require `LOADOUT_NON_INTERACTIVE=1` (the Justfile sets this).

## Architecture

| Module | Purpose |
|---|---|
| `src/cli/` | Clap derive parser — all commands, subcommands, flags |
| `src/config/` | `loadout.toml` serialization, XDG path resolution |
| `src/registry/` | Provenance tracking — which skill from which source (JSON) |
| `src/source/` | Fetch (git/HTTP/zip), detect structure, discover skills, normalize |
| `src/agent/` | Install/uninstall skills to agent directories, diff comparison |
| `src/bundle/` | Named groups of skills for batch operations |
| `src/marketplace.rs` | Marketplace manifest generation and operations |
| `src/prompt.rs` | Interactive prompts (dialoguer) |
| `src/output/` | Formatted terminal and JSON output |

### Source Detection Hierarchy

Loadout auto-detects what a URL points to. The hierarchy matters:
1. **Marketplace** — contains multiple plugins, each with skills
2. **Plugin** — a directory of related skills (has `skills/` subdirectory)
3. **Flat skills** — loose `.md` files auto-wrapped into a plugin
4. **Single skill** — one skill file

### Key Data Paths

- Config: `~/.local/share/loadout/loadout.toml`
- Registry: `~/.local/share/loadout/registry.json`
- Agents: agent-specific directories like `~/.claude/skills/`, `~/.codex/skills/`

## Test Infrastructure

Three testing tiers, each with a distinct purpose:

### 1. Rust Tests (`tests/*.rs`)

Library-level tests using `tempfile::TempDir` for isolation. Call module functions directly — no subprocess, no CLI binary.

- `smoke_test.rs` — basic sanity checks
- `cli_flags.rs` — clap argument parsing
- `functional_*.rs` — module-level behavior (source ops, agent ops, install, bundles, dry-run, etc.)
- `integration_*.rs` — cross-module workflows (config→source→agent→install)

Run with: `just test`

### 2. Bash Harness (`tests/harness/`)

Black-box CLI tests that invoke the built binary as a subprocess. Runs in Docker with no network access.

- **`lib.sh`** — assertion library: `assert_exit_code`, `assert_stdout_contains`, `assert_stderr_contains`, `assert_file_exists`, `assert_file_contains`, `assert_json_field`
- **`runner.sh`** — discovers and runs all `test_*` functions, resets environment between tests
- **`setup.sh`** — creates mock agent directories, sets env vars
- **`suite/`** — 12 numbered test files (`00_cli_framework.sh` through `11_end_to_end.sh`)

Run with: `just harness`

### 3. Docker Sandbox (`tests/sandbox/`)

Network-enabled functional tests that clone real git repos and test SSH workflows.

- **`run`** — container launcher with 3 modes: interactive, `--test`, `--keep-alive`
- **`suite/`** — 6 executable test scripts covering git sources, detection, install-to-agents, local clone, overwrite protection, glob filtering
- Uses the same assertion library from the harness

Run with: `just sandbox-test` or `just sandbox-keep` (for post-test exploration)

### Test Fixtures (`tests/fixtures/`)

Shared by all test tiers:
- `single-skill/` — one skill directory
- `plugin-source/` — multi-skill plugin
- `flat-skills/` — loose `.md` files (auto-wrapped)
- `full-source/` — multi-plugin marketplace
- `invalid/` — malformed inputs for error-path testing

## Conventions

- **Naming**: names describe what code does, not implementation details or history. No `ZodValidator`, `MCPWrapper`, `NewAPI` patterns.
- **Error handling**: `anyhow` with `.context()` for all fallible operations.
- **CLI output**: supports `--json`, `--quiet`, `--verbose`, `--dry-run` on all commands.
- **Non-interactive mode**: `LOADOUT_NON_INTERACTIVE=1` suppresses all prompts — required for CI and tests.
- **Test isolation**: every test gets a fresh `TempDir` or reset environment. Tests must not depend on ordering or shared state.
- **No test modification to pass**: fix the code, not the tests. If a test is wrong, discuss with Trent first.

## Editing Tests

When adding or modifying tests:

- **Rust tests**: follow existing patterns in the same file. Use `tempfile::TempDir`, build `Config`/`Registry` in memory, assert on return values and filesystem state.
- **Bash harness tests**: add `test_` functions to the appropriate numbered suite file. Use the assertion helpers from `lib.sh`. Each test function must be self-contained (environment resets between tests).
- **Sandbox tests**: only for scenarios requiring network (git clone, SSH). Keep these minimal — prefer harness tests for pure CLI behavior.
