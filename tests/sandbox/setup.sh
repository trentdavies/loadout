#!/usr/bin/env bash
# Environment setup and logging for sandbox functional tests.
# Source this file — do not execute directly.

# ---------------------------------------------------------------------------
# Assertion library
# ---------------------------------------------------------------------------
source /tests/harness/lib.sh

# ---------------------------------------------------------------------------
# Path configuration
# ---------------------------------------------------------------------------
export SKITTLE="${SKITTLE:-/usr/local/bin/skittle}"
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/sandbox-data}"

export SANDBOX_TARGET_CLAUDE="/tmp/sandbox-targets/claude"
export SANDBOX_TARGET_CODEX="/tmp/sandbox-targets/codex"
export SANDBOX_LOCAL="/tmp/sandbox-local"
export SANDBOX_LOG="${XDG_DATA_HOME}/sandbox.log"

# ---------------------------------------------------------------------------
# sandbox_init — init skittle + register both mock targets (idempotent)
# ---------------------------------------------------------------------------
sandbox_init() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" target add claude "$SANDBOX_TARGET_CLAUDE" --name sandbox-claude --scope machine --sync auto >/dev/null 2>&1
  "$SKITTLE" target add codex "$SANDBOX_TARGET_CODEX" --name sandbox-codex --scope machine --sync auto >/dev/null 2>&1
}

# ---------------------------------------------------------------------------
# Logging — writes to both terminal and $SANDBOX_LOG
# ---------------------------------------------------------------------------

# Ensure log directory exists
mkdir -p "$(dirname "$SANDBOX_LOG")" 2>/dev/null

log_section() {
  local title="$1"
  local line="=== $title ==="
  echo "" | tee -a "$SANDBOX_LOG"
  echo "$line" | tee -a "$SANDBOX_LOG"
  echo "" | tee -a "$SANDBOX_LOG"
}

log_cmd() {
  local cmd_display="$*"
  printf "  \$ %s\n" "$cmd_display" | tee -a "$SANDBOX_LOG"

  local output exit_code
  output=$("$@" 2>&1)
  exit_code=$?

  printf "  exit: %d\n" "$exit_code" | tee -a "$SANDBOX_LOG"

  # Show key output lines (first 10 non-empty lines)
  if [ -n "$output" ]; then
    echo "$output" | head -10 | while IFS= read -r line; do
      printf "  %s\n" "$line" | tee -a "$SANDBOX_LOG"
    done
  fi
  echo "" | tee -a "$SANDBOX_LOG"

  # Return the exit code so callers can check it
  return "$exit_code"
}

log_check() {
  local passed="$1"
  local description="$2"
  if [ "$passed" = "1" ] || [ "$passed" = "true" ]; then
    printf "  [CHECKED] %s\n" "$description" | tee -a "$SANDBOX_LOG"
  else
    printf "  [FAILED]  %s\n" "$description" | tee -a "$SANDBOX_LOG"
  fi
}
