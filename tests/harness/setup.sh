#!/usr/bin/env bash
# Environment setup for skittle CLI test harness.
# Source this file — do not execute directly.

# ---------------------------------------------------------------------------
# Path configuration
# ---------------------------------------------------------------------------
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export FIXTURES_DIR="$(cd "$HARNESS_DIR/../fixtures" && pwd)"

# Skittle binary — override via environment if needed
export SKITTLE="${SKITTLE:-/usr/local/bin/skittle}"

# XDG paths (set by Dockerfile, but default for local runs)
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-/tmp/test-config}"
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/test-data}"

# Mock target directories
export TARGET_CLAUDE="/tmp/test-targets/claude"
export TARGET_CODEX="/tmp/test-targets/codex"

# ---------------------------------------------------------------------------
# reset_environment
#   Wipe all skittle state and recreate empty mock targets.
#   Call at the start of each test function for isolation.
# ---------------------------------------------------------------------------
reset_environment() {
  # Wipe skittle config
  rm -rf "$XDG_CONFIG_HOME/skittle"

  # Wipe skittle data/cache
  rm -rf "$XDG_DATA_HOME/skittle"

  # Recreate empty mock target directories
  rm -rf "$TARGET_CLAUDE" "$TARGET_CODEX"
  mkdir -p "$TARGET_CLAUDE" "$TARGET_CODEX"
}

# ---------------------------------------------------------------------------
# setup_source_and_targets
#   Convenience helper: init skittle, add the plugin-source fixture,
#   and register claude + codex mock targets.
#   Use in tests that need a working baseline environment.
# ---------------------------------------------------------------------------
setup_source_and_targets() {
  reset_environment

  # Initialize skittle config
  "$SKITTLE" init >/dev/null 2>&1

  # Add the plugin-source fixture as a source
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1

  # Register mock targets
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$SKITTLE" target add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto >/dev/null 2>&1
}
