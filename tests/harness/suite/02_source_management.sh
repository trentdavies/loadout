#!/usr/bin/env bash
# Suite 02: Source Management
# Tests add (local + git), remove, update, duplicate name error.

test_source_add_local() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name test-plugin
  assert_stdout_contains "test-plugin" "$SKITTLE" list
}

test_source_add_local_full_source() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/full-source" --name test-source
  assert_stdout_contains "test-source" "$SKITTLE" list
}

test_source_add_git() {
  skip_if_no_network && return
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add https://github.com/anthropics/courses.git --name anthropic
  assert_stdout_contains "anthropic" "$SKITTLE" list
}

test_source_remove() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" remove test-plugin --force
  # Should no longer appear in list
  local output
  output=$("$SKITTLE" list 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin"; then
    _fail "source still listed after remove" "test-plugin absent" "still present"
  else
    _pass "source removed from list"
  fi
}

test_source_remove_preview_default() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$SKITTLE" remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "remove defaults to preview mode"
  else
    _fail "remove did not show preview" "would/force message" "$output"
  fi
  # Source should still be listed
  assert_stdout_contains "test-plugin" "$SKITTLE" list
}

test_source_update() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" update test-plugin
}

test_source_update_all() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name test-plugin >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" update
}

test_source_duplicate_name_error() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name dupe >/dev/null 2>&1
  # Adding with the same name again should error
  local output
  output=$("$SKITTLE" add "$FIXTURES_DIR/flat-skills" --name dupe 2>&1)
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
  output=$("$SKITTLE" remove test-plugin 2>&1)
  if echo "$output" | grep -qiE "warn|force|installed|would"; then
    _pass "remove warns about installed skills"
  else
    _fail "remove did not warn about installed skills" "warning or preview" "$output"
  fi
  # Source should still exist (no --force)
  assert_stdout_contains "test-plugin" "$SKITTLE" list
}
