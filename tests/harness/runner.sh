#!/usr/bin/env bash
# Test runner for skittle CLI test harness.
# Discovers suite/*.sh files, sources each, and executes all test_ functions.
#
# Usage:
#   runner.sh              # Run all suites
#   runner.sh --suite 02   # Run only suite 02_*

set -euo pipefail

HARNESS_DIR="$(cd "$(dirname "$0")" && pwd)"
SUITE_DIR="$HARNESS_DIR/suite"

# Source the assertion library
source "$HARNESS_DIR/lib.sh"

# Source environment setup
source "$HARNESS_DIR/setup.sh"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
SUITE_FILTER=""

while [ $# -gt 0 ]; do
  case "$1" in
    --suite)
      SUITE_FILTER="$2"
      shift 2
      ;;
    -h|--help|help)
      echo "Usage: runner.sh [--suite <number>]"
      echo ""
      echo "Options:"
      echo "  --suite <number>   Run only the suite matching this prefix (e.g., 02)"
      echo ""
      echo "Environment:"
      echo "  SKIP_NETWORK=1     Skip tests that require network access"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Check for skittle binary
# ---------------------------------------------------------------------------
if [ ! -x "$SKITTLE" ]; then
  echo ""
  echo "═══════════════════════════════════════════════"
  printf "\033[31mBUILD FAILED\033[0m — skittle binary not found at: %s\n" "$SKITTLE"
  echo "Build the project first: cargo build --release"
  echo "═══════════════════════════════════════════════"
  exit 2
fi

# ---------------------------------------------------------------------------
# Discover and run suites
# ---------------------------------------------------------------------------
SUITE_FILES=()

if [ -n "$SUITE_FILTER" ]; then
  # Match suites starting with the filter prefix
  for f in "$SUITE_DIR"/${SUITE_FILTER}_*.sh "$SUITE_DIR"/${SUITE_FILTER}.sh; do
    [ -f "$f" ] && SUITE_FILES+=("$f")
  done
  if [ ${#SUITE_FILES[@]} -eq 0 ]; then
    echo "No suite found matching: $SUITE_FILTER" >&2
    exit 1
  fi
else
  # All suites in alphanumeric order
  for f in "$SUITE_DIR"/*.sh; do
    [ -f "$f" ] && SUITE_FILES+=("$f")
  done
fi

if [ ${#SUITE_FILES[@]} -eq 0 ]; then
  echo "No test suites found in: $SUITE_DIR" >&2
  exit 1
fi

echo ""
echo "Skittle CLI Test Harness"
echo "═══════════════════════════════════════════════"
echo "Binary:  $SKITTLE"
echo "Suites:  ${#SUITE_FILES[@]}"
echo "Network: $([ "${SKIP_NETWORK:-0}" = "1" ] && echo "disabled" || echo "enabled")"
echo "═══════════════════════════════════════════════"

for suite_file in "${SUITE_FILES[@]}"; do
  suite_name="$(basename "$suite_file" .sh)"
  echo ""
  printf "── Suite: %s ──\n" "$suite_name"

  # Source the suite file (defines test_ functions)
  source "$suite_file"

  # Discover and run all test_ functions defined in this suite
  test_functions=$(declare -F | awk '{print $3}' | grep "^test_" | sort)

  for test_fn in $test_functions; do
    CURRENT_TEST="$suite_name::$test_fn"
    # Each test gets a clean environment
    reset_environment
    # Run the test
    "$test_fn"
  done

  # Unset test functions to avoid re-running them in next suite
  for test_fn in $test_functions; do
    unset -f "$test_fn"
  done
done

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print_summary

# Exit with appropriate code
if [ "$FAIL_COUNT" -gt 0 ]; then
  exit 1
fi
exit 0
