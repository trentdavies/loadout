#!/usr/bin/env bash
# Environment setup for loadout CLI test harness.
# Source this file — do not execute directly.

# ---------------------------------------------------------------------------
# Path configuration
# ---------------------------------------------------------------------------
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export FIXTURES_DIR="$(cd "$HARNESS_DIR/../fixtures" && pwd)"

# Loadout binary — override via environment if needed
export LOADOUT="${LOADOUT:-/usr/local/bin/loadout}"

# XDG path (set by Dockerfile, but default for local runs)
# Config + data both live under XDG_DATA_HOME/loadout/
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/test-data}"

# Mock agent directories
export TARGET_CLAUDE="/tmp/test-targets/claude"
export TARGET_CODEX="/tmp/test-targets/codex"

# ---------------------------------------------------------------------------
# reset_environment
#   Wipe all loadout state and recreate empty mock agents.
#   Call at the start of each test function for isolation.
# ---------------------------------------------------------------------------
reset_environment() {
  # Wipe all loadout state (config + data live together)
  rm -rf "$XDG_DATA_HOME/loadout"

  # Recreate empty mock agent directories
  rm -rf "$TARGET_CLAUDE" "$TARGET_CODEX"
  mkdir -p "$TARGET_CLAUDE" "$TARGET_CODEX"
}

# ---------------------------------------------------------------------------
# setup_source_and_agents
#   Convenience helper: init loadout, add the plugin-source fixture,
#   and register claude + codex mock agents.
#   Use in tests that need a working baseline environment.
# ---------------------------------------------------------------------------
setup_source_and_agents() {
  reset_environment

  # Initialize loadout config
  "$LOADOUT" init >/dev/null 2>&1

  # Add the plugin-source fixture as a source
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin >/dev/null 2>&1

  # Register mock agents
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" agent add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto >/dev/null 2>&1
}
