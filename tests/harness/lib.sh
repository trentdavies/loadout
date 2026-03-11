#!/usr/bin/env bash
# Assertion library for skittle CLI test harness.
# Source this file — do not execute directly.

# ---------------------------------------------------------------------------
# Counters
# ---------------------------------------------------------------------------
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
CURRENT_TEST=""

# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------
_pass() {
  PASS_COUNT=$((PASS_COUNT + 1))
  printf "  \033[32mPASS\033[0m %s — %s\n" "$CURRENT_TEST" "$1"
}

_fail() {
  FAIL_COUNT=$((FAIL_COUNT + 1))
  printf "  \033[31mFAIL\033[0m %s — %s\n" "$CURRENT_TEST" "$1"
  if [ -n "$2" ]; then
    printf "       expected: %s\n" "$2"
    printf "       actual:   %s\n" "$3"
  fi
}

_skip() {
  SKIP_COUNT=$((SKIP_COUNT + 1))
  printf "  \033[33mSKIP\033[0m %s — %s\n" "$CURRENT_TEST" "$1"
}

# ---------------------------------------------------------------------------
# assert_exit_code <expected> <command...>
#   Run command, check exit code.
# ---------------------------------------------------------------------------
assert_exit_code() {
  local expected="$1"; shift
  local actual
  "$@" >/dev/null 2>&1
  actual=$?
  if [ "$actual" -eq "$expected" ]; then
    _pass "exit code $expected"
  else
    _fail "exit code mismatch" "$expected" "$actual"
  fi
}

# ---------------------------------------------------------------------------
# assert_stdout_contains <pattern> <command...>
#   Run command, check stdout contains pattern (grep -qF).
# ---------------------------------------------------------------------------
assert_stdout_contains() {
  local pattern="$1"; shift
  local stdout
  stdout=$("$@" 2>/dev/null)
  if echo "$stdout" | grep -qF "$pattern"; then
    _pass "stdout contains '$pattern'"
  else
    _fail "stdout missing '$pattern'" "'$pattern' present" "not found in output"
  fi
}

# ---------------------------------------------------------------------------
# assert_stderr_contains <pattern> <command...>
#   Run command, check stderr contains pattern (grep -qF).
# ---------------------------------------------------------------------------
assert_stderr_contains() {
  local pattern="$1"; shift
  local stderr
  stderr=$("$@" 2>&1 1>/dev/null)
  if echo "$stderr" | grep -qF "$pattern"; then
    _pass "stderr contains '$pattern'"
  else
    _fail "stderr missing '$pattern'" "'$pattern' present" "not found in stderr"
  fi
}

# ---------------------------------------------------------------------------
# assert_stdout_eq <expected> <command...>
#   Run command, check exact stdout match (trimmed).
# ---------------------------------------------------------------------------
assert_stdout_eq() {
  local expected="$1"; shift
  local actual
  actual=$("$@" 2>/dev/null)
  if [ "$actual" = "$expected" ]; then
    _pass "stdout exact match"
  else
    _fail "stdout mismatch" "$expected" "$actual"
  fi
}

# ---------------------------------------------------------------------------
# assert_file_exists <path>
# ---------------------------------------------------------------------------
assert_file_exists() {
  if [ -f "$1" ]; then
    _pass "file exists: $1"
  else
    _fail "file missing" "$1 exists" "not found"
  fi
}

# ---------------------------------------------------------------------------
# assert_file_not_exists <path>
# ---------------------------------------------------------------------------
assert_file_not_exists() {
  if [ ! -e "$1" ]; then
    _pass "file does not exist: $1"
  else
    _fail "file should not exist" "$1 absent" "found"
  fi
}

# ---------------------------------------------------------------------------
# assert_dir_exists <path>
# ---------------------------------------------------------------------------
assert_dir_exists() {
  if [ -d "$1" ]; then
    _pass "dir exists: $1"
  else
    _fail "dir missing" "$1 exists" "not found"
  fi
}

# ---------------------------------------------------------------------------
# assert_file_contains <path> <pattern>
#   Check file content contains pattern (grep -qF).
# ---------------------------------------------------------------------------
assert_file_contains() {
  local path="$1"
  local pattern="$2"
  if [ ! -f "$path" ]; then
    _fail "file missing for content check" "$path exists" "not found"
    return
  fi
  if grep -qF "$pattern" "$path"; then
    _pass "file '$path' contains '$pattern'"
  else
    _fail "file '$path' missing content" "'$pattern' present" "not found"
  fi
}

# ---------------------------------------------------------------------------
# assert_json_field <json_string> <jq_path> <expected>
#   Parse JSON, check field value via jq.
# ---------------------------------------------------------------------------
assert_json_field() {
  local json="$1"
  local jq_path="$2"
  local expected="$3"
  local actual
  actual=$(echo "$json" | jq -r "$jq_path" 2>/dev/null)
  if [ "$actual" = "$expected" ]; then
    _pass "json $jq_path == '$expected'"
  else
    _fail "json field mismatch at $jq_path" "$expected" "$actual"
  fi
}

# ---------------------------------------------------------------------------
# skip_if_no_network
#   Call at the top of a test function to skip if SKIP_NETWORK=1.
#   Returns 0 if skipped (caller should return), 1 if network is available.
# ---------------------------------------------------------------------------
skip_if_no_network() {
  if [ "${SKIP_NETWORK:-0}" = "1" ]; then
    _skip "network required (SKIP_NETWORK=1)"
    return 0
  fi
  return 1
}

# ---------------------------------------------------------------------------
# Summary — call at the end of the test run.
# ---------------------------------------------------------------------------
print_summary() {
  local total=$((PASS_COUNT + FAIL_COUNT + SKIP_COUNT))
  echo ""
  echo "═══════════════════════════════════════════════"
  printf "Results: \033[32m%d passed\033[0m, \033[31m%d failed\033[0m, \033[33m%d skipped\033[0m, %d total\n" \
    "$PASS_COUNT" "$FAIL_COUNT" "$SKIP_COUNT" "$total"
  echo "═══════════════════════════════════════════════"
}
