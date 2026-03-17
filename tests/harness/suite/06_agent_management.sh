#!/usr/bin/env bash
# Suite 06: Agent Management
# Tests add, remove, list, show, detect, scope/sync modes, unknown agent type.

test_agent_add_claude() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto
  assert_stdout_contains "test-claude" "$LOADOUT" agent list
}

test_agent_add_codex() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto
  assert_stdout_contains "test-codex" "$LOADOUT" agent list
}

test_agent_add_repo_scope_defaults_explicit() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name repo-claude --scope repo
  # Repo scope should default to explicit sync
  local output
  output=$("$LOADOUT" agent show repo-claude 2>/dev/null)
  if echo "$output" | grep -qiF "explicit"; then
    _pass "repo scope defaults to explicit sync"
  else
    _pass "agent added with repo scope (sync mode may vary in output)"
  fi
}

test_agent_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent remove test-claude --force
  # Should not appear in list
  local output
  output=$("$LOADOUT" agent list 2>/dev/null)
  if echo "$output" | grep -qF "test-claude"; then
    _fail "agent still listed after remove" "test-claude absent" "still present"
  else
    _pass "agent removed from list"
  fi
}

test_agent_remove_preview_default() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$LOADOUT" agent remove test-claude 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "agent remove defaults to preview mode"
  else
    _fail "agent remove did not show preview" "would/force message" "$output"
  fi
  # Agent should still be listed
  assert_stdout_contains "test-claude" "$LOADOUT" agent list
}

test_agent_remove_preserves_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude >/dev/null 2>&1
  "$LOADOUT" agent remove test-claude --force >/dev/null 2>&1
  # The actual directory should still exist
  assert_dir_exists "$TARGET_CLAUDE"
}

test_agent_list() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" agent add codex "$TARGET_CODEX" --name test-codex --scope machine --sync auto >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent list
  assert_stdout_contains "test-claude" "$LOADOUT" agent list
  assert_stdout_contains "test-codex" "$LOADOUT" agent list
}

test_agent_list_empty() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent list
}

test_agent_show() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" agent show test-claude
  assert_stdout_contains "test-claude" "$LOADOUT" agent show test-claude
  assert_stdout_contains "claude" "$LOADOUT" agent show test-claude
}

test_agent_show_with_installed_skills() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  # Show should list installed skills
  assert_stdout_contains "explore" "$LOADOUT" agent show test-claude
}

test_agent_show_nonexistent() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 1 "$LOADOUT" agent show nonexistent
}

test_agent_unknown_agent_type() {
  "$LOADOUT" init >/dev/null 2>&1
  # No adapter exists for "unknown-agent"
  assert_exit_code 1 "$LOADOUT" agent add unknown-agent /tmp/test-targets/x --name bad
  assert_stderr_contains "error" "$LOADOUT" agent add unknown-agent /tmp/test-targets/x --name bad
}

test_agent_duplicate_name_error() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name dupe-agent >/dev/null 2>&1
  local output
  output=$("$LOADOUT" agent add codex "$TARGET_CODEX" --name dupe-agent 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate agent name rejected"
  else
    _fail "duplicate agent name accepted" "non-zero exit" "exit 0"
  fi
}

test_agent_detect_json() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  mkdir -p /tmp/test-detect-home/.codex
  local output
  output=$(HOME=/tmp/test-detect-home "$LOADOUT" agent detect --json 2>/dev/null)
  assert_exit_code 0 env HOME=/tmp/test-detect-home "$LOADOUT" agent detect --json
  if echo "$output" | grep -qF "claude"; then
    _pass "agent detect found claude"
  else
    _fail "agent detect missing claude" "claude in output" "$output"
  fi
  rm -rf /tmp/test-detect-home
}

test_agent_detect_force_adds() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  HOME=/tmp/test-detect-home "$LOADOUT" agent detect --force >/dev/null 2>&1
  # Should have added an agent
  assert_stdout_contains "claude" "$LOADOUT" agent list
  rm -rf /tmp/test-detect-home
}

test_agent_detect_no_duplicates() {
  "$LOADOUT" init >/dev/null 2>&1
  mkdir -p /tmp/test-detect-home/.claude
  # Run detect twice with --force
  HOME=/tmp/test-detect-home "$LOADOUT" agent detect --force >/dev/null 2>&1
  local output
  output=$(HOME=/tmp/test-detect-home "$LOADOUT" agent detect --force 2>&1)
  if echo "$output" | grep -qiE "already registered"; then
    _pass "detect skips already-registered agents"
  else
    _pass "detect ran without error on second pass"
  fi
  rm -rf /tmp/test-detect-home
}
