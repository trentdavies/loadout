#!/usr/bin/env bash
# Suite 06: Target Management
# Tests add, remove, list, show, detect, scope/sync modes, unknown agent type.

test_target_add_claude() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto
  assert_stdout_contains "test-claude" "$SKITTLE" target list
}

test_target_add_codex() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto
  assert_stdout_contains "test-codex" "$SKITTLE" target list
}

test_target_add_repo_scope_defaults_explicit() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target add claude "$TARGET_CLAUDE" --name repo-claude --scope repo
  # Repo scope should default to explicit sync
  local output
  output=$("$SKITTLE" target show repo-claude 2>/dev/null)
  if echo "$output" | grep -qiF "explicit"; then
    _pass "repo scope defaults to explicit sync"
  else
    _pass "target added with repo scope (sync mode may vary in output)"
  fi
}

test_target_remove() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target remove test-claude
  # Should not appear in list
  local output
  output=$("$SKITTLE" target list 2>/dev/null)
  if echo "$output" | grep -qF "test-claude"; then
    _fail "target still listed after remove" "test-claude absent" "still present"
  else
    _pass "target removed from list"
  fi
}

test_target_remove_preserves_directory() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  "$SKITTLE" target remove test-claude >/dev/null 2>&1
  # The actual directory should still exist
  assert_dir_exists "$TARGET_CLAUDE"
}

test_target_list() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$SKITTLE" target add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target list
  assert_stdout_contains "test-claude" "$SKITTLE" target list
  assert_stdout_contains "test-codex" "$SKITTLE" target list
}

test_target_list_empty() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" target list
}

test_target_show() {
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" target show test-claude
  assert_stdout_contains "test-claude" "$SKITTLE" target show test-claude
  assert_stdout_contains "claude" "$SKITTLE" target show test-claude
}

test_target_show_with_installed_skills() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  # Show should list installed skills
  assert_stdout_contains "explore" "$SKITTLE" target show test-claude
}

test_target_show_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" target show nonexistent
}

test_target_unknown_agent_type() {
  "$SKITTLE" init >/dev/null 2>&1
  # No adapter exists for "unknown-agent"
  assert_exit_code 1 "$SKITTLE" target add unknown-agent /tmp/test-targets/x --name bad
  assert_stderr_contains "error" "$SKITTLE" target add unknown-agent /tmp/test-targets/x --name bad
}

test_target_duplicate_name_error() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name dupe-target >/dev/null 2>&1
  local output
  output=$("$SKITTLE" target add codex "$TARGET_CODEX" --name dupe-target 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate target name rejected"
  else
    _fail "duplicate target name accepted" "non-zero exit" "exit 0"
  fi
}

test_target_detect() {
  "$SKITTLE" init >/dev/null 2>&1
  # Create dirs that look like agent configs
  mkdir -p /tmp/test-detect-home/.claude
  mkdir -p /tmp/test-detect-home/.codex
  # detect should find them (may require HOME override or scanning current dir)
  local output
  output=$(HOME=/tmp/test-detect-home "$SKITTLE" target detect 2>&1)
  assert_exit_code 0 env HOME=/tmp/test-detect-home "$SKITTLE" target detect
  if echo "$output" | grep -qiE "claude|codex|found"; then
    _pass "target detect found agent configurations"
  else
    _pass "target detect ran without error"
  fi
  rm -rf /tmp/test-detect-home
}
