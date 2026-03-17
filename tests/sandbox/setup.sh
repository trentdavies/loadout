#!/usr/bin/env bash
# Environment setup and logging for sandbox functional tests.
# Source this file — do not execute directly.

# ---------------------------------------------------------------------------
# Assertion library
# ---------------------------------------------------------------------------
source /tests/harness/lib.sh

# ---------------------------------------------------------------------------
# Path configuration — mirrors a real user's filesystem
# ---------------------------------------------------------------------------
export LOADOUT="${LOADOUT:-$HOME/.local/bin/equip}"
export NO_COLOR=1
export EQUIP_NON_INTERACTIVE=1
export XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

export SANDBOX_TARGET_CLAUDE="$HOME/.claude"
export SANDBOX_TARGET_CODEX="$HOME/.codex"
export SANDBOX_LOCAL="$HOME/repos"
export SANDBOX_LOG="$XDG_DATA_HOME/equip/sandbox.log"

# ---------------------------------------------------------------------------
# sandbox_init — init equip + register both mock agents (idempotent)
# ---------------------------------------------------------------------------
sandbox_init() {
  mkdir -p "$SANDBOX_LOCAL"
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$SANDBOX_TARGET_CLAUDE" --name sandbox-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" agent add codex "$SANDBOX_TARGET_CODEX" --name sandbox-codex --scope machine --sync auto >/dev/null 2>&1
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
