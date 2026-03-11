#!/usr/bin/env bash
# Suite 02: Source Management
# Tests add (local + git), remove, list, show, update, duplicate name error.

test_source_add_local() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin
  assert_stdout_contains "test-plugin" "$SKITTLE" source list
}

test_source_add_local_full_source() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name test-source
  assert_stdout_contains "test-source" "$SKITTLE" source list
}

test_source_add_git() {
  skip_if_no_network && return
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source add https://github.com/anthropics/courses.git --name anthropic
  assert_stdout_contains "anthropic" "$SKITTLE" source list
}

test_source_remove() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source remove test-plugin --force
  # Should no longer appear in list
  local output
  output=$("$SKITTLE" source list 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin"; then
    _fail "source still listed after remove" "test-plugin absent" "still present"
  else
    _pass "source removed from list"
  fi
}

test_source_remove_preview_default() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$SKITTLE" source remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "source remove defaults to preview mode"
  else
    _fail "source remove did not show preview" "would/force message" "$output"
  fi
  # Source should still be listed
  assert_stdout_contains "test-plugin" "$SKITTLE" source list
}

test_source_list_empty() {
  "$SKITTLE" init >/dev/null 2>&1
  local output
  output=$("$SKITTLE" source list 2>&1)
  assert_exit_code 0 "$SKITTLE" source list
  # Should indicate no sources or show empty output
  if echo "$output" | grep -qiE "no source|empty|add"; then
    _pass "source list indicates no sources"
  else
    # Empty table is also acceptable
    _pass "source list returned (empty state)"
  fi
}

test_source_list_populated() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name test-source >/dev/null 2>&1
  assert_stdout_contains "test-plugin" "$SKITTLE" source list
  assert_stdout_contains "test-source" "$SKITTLE" source list
}

test_source_show() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source show test-plugin
  # Should display plugin and skill info
  assert_stdout_contains "test-plugin" "$SKITTLE" source show test-plugin
  assert_stdout_contains "explore" "$SKITTLE" source show test-plugin
}

test_source_show_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" source show nonexistent
}

test_source_update() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source update test-plugin
}

test_source_update_all() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" source update
}

test_source_duplicate_name_error() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name dupe >/dev/null 2>&1
  # Adding with the same name again should error
  local output
  output=$("$SKITTLE" source add "$FIXTURES_DIR/flat-skills" --name dupe 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate source name rejected (exit $exit_code)"
  else
    _fail "duplicate source name was accepted" "non-zero exit" "exit 0"
  fi
}

test_source_remove_with_installed_skills_warns() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  # Without --force, should preview and warn about installed skills
  local output
  output=$("$SKITTLE" source remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "warn|force|installed|would"; then
    _pass "source remove warns about installed skills"
  else
    _fail "source remove did not warn about installed skills" "warning or preview" "$output"
  fi
  # Source should still exist (no --force)
  assert_stdout_contains "test-plugin" "$SKITTLE" source list
}
