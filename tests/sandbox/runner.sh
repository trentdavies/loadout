#!/usr/bin/env bash
# Sandbox test runner — cumulative, no reset between tests.
# Suites build on each other: 01 adds sources, 02 checks detection, etc.
# After all suites, keeps the container alive for exploration.

set -uo pipefail

SANDBOX_DIR="$(cd "$(dirname "$0")" && pwd)"
SUITE_DIR="$SANDBOX_DIR/suite"

# Source setup (which also sources lib.sh)
source "$SANDBOX_DIR/setup.sh"

# ---------------------------------------------------------------------------
# Check for skittle binary
# ---------------------------------------------------------------------------
if [ ! -x "$SKITTLE" ]; then
  echo ""
  echo "═══════════════════════════════════════════════"
  printf "\033[31mBUILD FAILED\033[0m — skittle binary not found at: %s\n" "$SKITTLE"
  echo "═══════════════════════════════════════════════"
  exit 2
fi

# ---------------------------------------------------------------------------
# Initialize sandbox environment (once, cumulative)
# ---------------------------------------------------------------------------
sandbox_init

echo "" | tee -a "$SANDBOX_LOG"
echo "Skittle Sandbox — Functional Tests" | tee -a "$SANDBOX_LOG"
echo "═══════════════════════════════════════════════" | tee -a "$SANDBOX_LOG"
echo "Binary:  $SKITTLE" | tee -a "$SANDBOX_LOG"
echo "Data:    $XDG_DATA_HOME" | tee -a "$SANDBOX_LOG"
echo "Targets: $SANDBOX_TARGET_CLAUDE, $SANDBOX_TARGET_CODEX" | tee -a "$SANDBOX_LOG"
echo "Log:     $SANDBOX_LOG" | tee -a "$SANDBOX_LOG"
echo "═══════════════════════════════════════════════" | tee -a "$SANDBOX_LOG"

# ---------------------------------------------------------------------------
# Discover and run suites in order
# ---------------------------------------------------------------------------
SUITE_FILES=()
for f in "$SUITE_DIR"/*.sh; do
  [ -f "$f" ] && SUITE_FILES+=("$f")
done

if [ ${#SUITE_FILES[@]} -eq 0 ]; then
  echo "No test suites found in: $SUITE_DIR" >&2
  exit 1
fi

for suite_file in "${SUITE_FILES[@]}"; do
  suite_name="$(basename "$suite_file" .sh)"

  log_section "Suite: $suite_name"
  printf "── Suite: %s ──\n" "$suite_name"

  # Source the suite (defines test_ functions)
  source "$suite_file"

  # Discover and run test_ functions in declaration order
  test_functions=$(declare -F | awk '{print $3}' | grep "^test_" | sort)

  for test_fn in $test_functions; do
    CURRENT_TEST="$suite_name::$test_fn"
    # No reset_environment — cumulative by design
    "$test_fn"
  done

  # Unset test functions to avoid re-running in next suite
  for test_fn in $test_functions; do
    unset -f "$test_fn"
  done
done

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print_summary

echo "" | tee -a "$SANDBOX_LOG"
echo "Full log: $SANDBOX_LOG" | tee -a "$SANDBOX_LOG"
echo "Container will stay alive for exploration." | tee -a "$SANDBOX_LOG"
echo "  cat $SANDBOX_LOG" | tee -a "$SANDBOX_LOG"
echo "  tree $XDG_DATA_HOME/skittle/" | tee -a "$SANDBOX_LOG"
echo "  skittle list" | tee -a "$SANDBOX_LOG"
echo "  skittle status" | tee -a "$SANDBOX_LOG"

# Keep container alive for exploration
exec sleep infinity
