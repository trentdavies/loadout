#!/usr/bin/env bash
# Suite 02: Source Management
# Tests add (local + git), remove, update, duplicate name error.

test_source_add_local() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin
  assert_stdout_contains "test-plugin" "$LOADOUT" list
}

test_source_add_local_full_source() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/full-source" --source test-source
  assert_stdout_contains "test-source" "$LOADOUT" list
}

test_source_add_git() {
  skip_if_no_network && return
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add https://github.com/anthropics/courses.git --source anthropic
  assert_stdout_contains "anthropic" "$LOADOUT" list
}

test_source_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" remove test-plugin --force
  # Should no longer appear in list
  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin"; then
    _fail "source still listed after remove" "test-plugin absent" "still present"
  else
    _pass "source removed from list"
  fi
}

test_source_remove_preview_default() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$LOADOUT" remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "remove defaults to preview mode"
  else
    _fail "remove did not show preview" "would/force message" "$output"
  fi
  # Source should still be listed
  assert_stdout_contains "test-plugin" "$LOADOUT" list
}

test_source_update() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" update test-plugin
}

test_source_update_all() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" update
}

test_source_duplicate_name_error() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source dupe >/dev/null 2>&1
  # Adding with the same name again should error
  local output
  output=$("$LOADOUT" add "$FIXTURES_DIR/flat-skills" --source dupe 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate source name rejected (exit $exit_code)"
  else
    _fail "duplicate source name was accepted" "non-zero exit" "exit 0"
  fi
}

test_source_remove_with_installed_skills_warns() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  # Without --force, should preview and warn about installed skills
  local output
  output=$("$LOADOUT" remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "warn|force|installed|would"; then
    _pass "remove warns about installed skills"
  else
    _fail "remove did not warn about installed skills" "warning or preview" "$output"
  fi
  # Source should still exist (no --force)
  assert_stdout_contains "test-plugin" "$LOADOUT" list
}
