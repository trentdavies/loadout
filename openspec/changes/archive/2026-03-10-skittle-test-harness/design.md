## Context

The `skittle-cli-v1` change specifies 11 capabilities and 49 tasks for a Rust CLI that manages agent skills. Before implementing any of it, we need an external test harness that exercises the compiled `skittle` binary as a black box. Tests must run in Docker for complete isolation — no host XDG paths, no real agent configs, no side effects.

The harness serves as the "red" in red-green development: every test fails initially because skittle doesn't exist yet. As implementation progresses, tests turn green.

## Goals / Non-Goals

**Goals:**
- Black-box CLI testing: invoke `skittle` as a subprocess, assert on exit codes, stdout/stderr content, and filesystem side effects
- Docker isolation: all tests run inside a container with controlled XDG paths and mock target directories
- Fixture-based: local test fixtures cover every source structure type (single file, flat dir, plugin, full source)
- Remote source testing: clone a real git repo (Anthropic skills) to validate git-based source fetching
- 1:1 mapping to specs: test suites correspond directly to the 11 capability specs in `skittle-cli-v1`
- Fast iteration: `docker build && docker run` to execute the full suite
- Clear output: each test prints PASS/FAIL with context on failure

**Non-Goals:**
- Unit testing Rust internals (that's in `skittle-cli-v1` task group 12)
- Integration testing of library APIs
- Performance or load testing
- Testing on multiple platforms (Linux container only for now)
- Testing adapters beyond claude and codex

## Decisions

### Test language: shell (bash)

Tests are plain bash scripts using a minimal assertion library (`assert_eq`, `assert_contains`, `assert_exit_code`, `assert_file_exists`, `assert_file_contains`). Each test is a function. The runner sources all test files and executes them in order.

Alternative considered: Rust integration tests (`cargo test`). Rejected — those test the library, not the installed binary. We want to test the actual CLI as a user would invoke it. Also, bash tests can run before any Rust code exists (they just all fail).

Alternative considered: Python with pytest. Rejected — adds a runtime dependency. Bash is available everywhere and maps naturally to CLI testing.

### Directory structure

```
tests/
├── harness/
│   ├── runner.sh              # Test runner: discovers and runs test files
│   ├── lib.sh                 # Assertion library + helpers
│   ├── suite/
│   │   ├── 00_cli_framework.sh
│   │   ├── 01_config.sh
│   │   ├── 02_source_management.sh
│   │   ├── 03_source_detection.sh
│   │   ├── 04_plugin_system.sh
│   │   ├── 05_local_registry.sh
│   │   ├── 06_target_management.sh
│   │   ├── 07_target_adapters.sh
│   │   ├── 08_skill_operations.sh
│   │   ├── 09_install_engine.sh
│   │   ├── 10_bundle_management.sh
│   │   └── 11_end_to_end.sh
│   └── setup.sh               # Per-suite setup: clean XDG, reset targets
├── fixtures/
│   ├── single-skill/
│   │   └── SKILL.md            # Minimal valid skill (single file source)
│   ├── flat-skills/
│   │   ├── explore/
│   │   │   └── SKILL.md
│   │   └── apply/
│   │       └── SKILL.md
│   ├── plugin-source/
│   │   ├── plugin.toml
│   │   └── skills/
│   │       ├── explore/
│   │       │   └── SKILL.md
│   │       ├── apply/
│   │       │   ├── SKILL.md
│   │       │   └── scripts/
│   │       │       └── run.sh
│   │       └── verify/
│   │           └── SKILL.md
│   └── full-source/
│       ├── source.toml
│       ├── test-plugin-a/
│       │   ├── plugin.toml
│       │   └── skills/
│       │       ├── skill-one/
│       │       │   └── SKILL.md
│       │       └── skill-two/
│       │           └── SKILL.md
│       └── test-plugin-b/
│           ├── plugin.toml
│           └── skills/
│               └── skill-three/
│                   └── SKILL.md
└── Dockerfile
```

### Docker setup

The Dockerfile:
1. Starts from `rust:latest` (to build skittle)
2. Copies the full project source
3. Runs `cargo build --release` (will fail until code exists — that's expected, so we handle this gracefully)
4. Sets up controlled XDG paths:
   - `XDG_CONFIG_HOME=/tmp/test-config`
   - `XDG_DATA_HOME=/tmp/test-data`
5. Creates mock target directories:
   - `/tmp/test-targets/claude/` (mock `.claude/`)
   - `/tmp/test-targets/codex/` (mock `.codex/`)
6. Copies test fixtures into the container
7. Runs the test harness

The build step is split from the test step using multi-stage or a wrapper script so that:
- If the build fails, the harness reports "BUILD FAILED: 0/N tests run" instead of crashing
- If the build succeeds, the harness runs all tests

### Assertion library (lib.sh)

```bash
assert_exit_code <expected> <command...>    # Run command, check exit code
assert_stdout_contains <pattern> <cmd...>   # Run command, check stdout contains
assert_stderr_contains <pattern> <cmd...>   # Run command, check stderr contains
assert_stdout_eq <expected> <cmd...>        # Run command, check exact stdout
assert_file_exists <path>                   # Check file exists
assert_file_not_exists <path>               # Check file doesn't exist
assert_file_contains <path> <pattern>       # Check file content matches
assert_dir_exists <path>                    # Check directory exists
assert_json_field <json> <jq-path> <expected>  # Parse JSON, check field value
```

Each assertion increments a global pass/fail counter. On failure, it prints the test name, expected value, actual value, and continues (does not abort the suite).

### Test isolation

Each test suite file gets a fresh environment via `setup.sh`:
- Wipe `$XDG_CONFIG_HOME/skittle/`
- Wipe `$XDG_DATA_HOME/skittle/`
- Wipe mock target directories and recreate empty ones
- This ensures tests don't depend on ordering (though suites are numbered for readability)

### Remote source testing

One test in `02_source_management.sh` clones the Anthropic skills repo (`https://github.com/anthropics/courses.git` or the actual skills repo URL — to be confirmed). This test is tagged as `@network` so it can be skipped in offline runs via `SKIP_NETWORK=1`.

### Suite-to-spec mapping

| Suite File | Capability Spec | Key Tests |
|---|---|---|
| `00_cli_framework.sh` | cli-framework | help flags, unknown commands, exit codes, global flags |
| `01_config.sh` | config-management | init, config show, config show --json, cache clean/show |
| `02_source_management.sh` | source-management | add (local + git), remove, list, show, update |
| `03_source_detection.sh` | source-detection | single file, flat dir, plugin.toml, source.toml, bad dir |
| `04_plugin_system.sh` | plugin-system | plugin list, plugin show, implicit plugins, plugin.toml parsing |
| `05_local_registry.sh` | local-registry | XDG paths, registry.json content, cache structure, skill identity resolution |
| `06_target_management.sh` | target-management | add, remove, list, show, detect, scope/sync modes |
| `07_target_adapters.sh` | target-adapters | claude adapter, codex adapter, custom TOML adapter, unknown format error |
| `08_skill_operations.sh` | skill-operations | skill list, skill show, filters, Agent Skills spec validation |
| `09_install_engine.sh` | install-engine | install --all/--skill/--plugin/--bundle, uninstall, dry run, idempotency |
| `10_bundle_management.sh` | bundle-management | create, delete, list, show, add, drop, swap, active tracking |
| `11_end_to_end.sh` | (all) | Full workflow: init → source add → target add → install --all → status → uninstall → cache clean |

### Invocation

```bash
# Build and run all tests
docker build -t skittle-test -f tests/Dockerfile . && docker run --rm skittle-test

# Run specific suite
docker run --rm skittle-test bash tests/harness/runner.sh --suite 02

# Skip network tests
docker run --rm -e SKIP_NETWORK=1 skittle-test
```

## Risks / Trade-offs

- **Build failure handling** → The harness must gracefully handle `cargo build` failing (which it will initially). The wrapper script checks for the binary before running tests and reports accordingly.
- **Network dependency** → Remote source tests require internet. Mitigated by `SKIP_NETWORK` flag and caching the clone in a Docker layer.
- **Bash test fragility** → Shell-based assertions are less robust than a real test framework. Mitigated by keeping the assertion library simple and well-tested itself.
- **Test maintenance** → Tests mirror specs 1:1. If specs change, tests must update. This is intentional — the harness IS the executable specification.
- **Docker build time** → Rust compilation is slow. Mitigated by using Docker layer caching for `Cargo.toml` / dependency downloads before copying source.
