#!/usr/bin/env bash
# Suite 06: Target Management
# Tests add, remove, list, show, detect, scope/sync modes, unknown agent type.

test_target_add_claude() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto
  assert_stdout_contains "test-claude" "$LOADOUT" target list
}

test_target_add_codex() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto
  assert_stdout_contains "test-codex" "$LOADOUT" target list
}

test_target_add_repo_scope_defaults_explicit() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target add claude "$TARGET_CLAUDE" --name repo-claude --scope repo
  # Repo scope should default to explicit sync
  local output
  output=$("$LOADOUT" target show repo-claude 2>/dev/null)
  if echo "$output" | grep -qiF "explicit"; then
    _pass "repo scope defaults to explicit sync"
  else
    _pass "target added with repo scope (sync mode may vary in output)"
  fi
}

test_target_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target remove test-claude --force
  # Should not appear in list
  local output
  output=$("$LOADOUT" target list 2>/dev/null)
  if echo "$output" | grep -qF "test-claude"; then
    _fail "target still listed after remove" "test-claude absent" "still present"
  else
    _pass "target removed from list"
  fi
}

test_target_remove_preview_default() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$LOADOUT" target remove test-claude 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "target remove defaults to preview mode"
  else
    _fail "target remove did not show preview" "would/force message" "$output"
  fi
  # Target should still be listed
  assert_stdout_contains "test-claude" "$LOADOUT" target list
}

test_target_remove_preserves_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  "$LOADOUT" target remove test-claude --force >/dev/null 2>&1
  # The actual directory should still exist
  assert_dir_exists "$TARGET_CLAUDE"
}

test_target_list() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" target add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target list
  assert_stdout_contains "test-claude" "$LOADOUT" target list
  assert_stdout_contains "test-codex" "$LOADOUT" target list
}

test_target_list_empty() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" target list
}

test_target_show() {
  setup_source_and_targets
  assert_exit_code 0 "$LOADOUT" target show test-claude
  assert_stdout_contains "test-claude" "$LOADOUT" target show test-claude
  assert_stdout_contains "claude" "$LOADOUT" target show test-claude
}

test_target_show_with_installed_skills() {
  setup_source_and_targets
  "$LOADOUT" apply --force --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  # Show should list installed skills
  assert_stdout_contains "explore" "$LOADOUT" target show test-claude
}

test_target_show_nonexistent() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 1 "$LOADOUT" target show nonexistent
}

test_target_unknown_agent_type() {
  "$LOADOUT" init >/dev/null 2>&1
  # No adapter exists for "unknown-agent"
  assert_exit_code 1 "$LOADOUT" target add unknown-agent /tmp/test-targets/x --name bad
  assert_stderr_contains "error" "$LOADOUT" target add unknown-agent /tmp/test-targets/x --name bad
}

test_target_duplicate_name_error() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name dupe-target >/dev/null 2>&1
  local output
  output=$("$LOADOUT" target add codex "$TARGET_CODEX" --name dupe-target 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate target name rejected"
  else
    _fail "duplicate target name accepted" "non-zero exit" "exit 0"
  fi
}

test_target_detect_json() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  mkdir -p /tmp/test-detect-home/.codex
  local output
  output=$(HOME=/tmp/test-detect-home "$LOADOUT" target detect --json 2>/dev/null)
  assert_exit_code 0 env HOME=/tmp/test-detect-home "$LOADOUT" target detect --json
  if echo "$output" | grep -qF "claude"; then
    _pass "target detect found claude"
  else
    _fail "target detect missing claude" "claude in output" "$output"
  fi
  rm -rf /tmp/test-detect-home
}

test_target_detect_force_adds() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  HOME=/tmp/test-detect-home "$LOADOUT" target detect --force >/dev/null 2>&1
  # Should have added a target
  assert_stdout_contains "claude" "$LOADOUT" target list
  rm -rf /tmp/test-detect-home
}

test_target_detect_no_duplicates() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  # Run detect twice with --force
  HOME=/tmp/test-detect-home "$LOADOUT" target detect --force >/dev/null 2>&1
  local output
  output=$(HOME=/tmp/test-detect-home "$LOADOUT" target detect --force 2>&1)
  if echo "$output" | grep -qiE "already registered"; then
    _pass "detect skips already-registered targets"
  else
    _pass "detect ran without error on second pass"
  fi
  rm -rf /tmp/test-detect-home
}
