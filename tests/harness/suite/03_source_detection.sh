#!/usr/bin/env bash
# Suite 03: Source Detection
# Tests progressive detection: single file, flat dir, plugin dir, full source,
# unrecognizable dir error, invalid skill warnings.

test_detect_single_file() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/single-skill/SKILL.md" --name single
  # Should result in one skill
  local output
  output=$("$SKITTLE" list 2>/dev/null)
  if echo "$output" | grep -qF "single-skill"; then
    _pass "single file detected as skill"
  else
    _fail "single file not detected" "single-skill in list" "$output"
  fi
}

test_detect_flat_directory() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/flat-skills" --name flat
  # Should detect 2 skills: explore and apply
  assert_stdout_contains "explore" "$SKITTLE" list
  assert_stdout_contains "apply" "$SKITTLE" list
}

test_detect_plugin_directory() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name plugged
  # Should detect plugin with 3 skills
  assert_stdout_contains "test-plugin" "$SKITTLE" list
  assert_stdout_contains "explore" "$SKITTLE" list
  assert_stdout_contains "apply" "$SKITTLE" list
  assert_stdout_contains "verify" "$SKITTLE" list
}

test_detect_full_source() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/full-source" --name full
  # Should detect 2 plugins (visible in list table columns)
  assert_stdout_contains "test-plugin-a" "$SKITTLE" list
  assert_stdout_contains "test-plugin-b" "$SKITTLE" list
  # Should detect 3 skills total
  assert_stdout_contains "skill-one" "$SKITTLE" list
  assert_stdout_contains "skill-two" "$SKITTLE" list
  assert_stdout_contains "skill-three" "$SKITTLE" list
}

test_detect_unrecognizable_directory() {
  "$SKITTLE" init >/dev/null 2>&1
  # empty-dir has no SKILL.md, no toml — should fail
  assert_exit_code 1 "$SKITTLE" add "$FIXTURES_DIR/invalid/empty-dir" --name bad
  assert_stderr_contains "error" "$SKITTLE" add "$FIXTURES_DIR/invalid/empty-dir" --name bad
}

test_detect_invalid_no_frontmatter_warns() {
  "$SKITTLE" init >/dev/null 2>&1
  # Adding a source that contains no-frontmatter skill should warn and skip it
  local output
  output=$("$SKITTLE" add "$FIXTURES_DIR/invalid/no-frontmatter/SKILL.md" --name nofm 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "warn|skip|invalid|frontmatter"; then
    _pass "no-frontmatter skill triggers warning or rejection"
  else
    _fail "no-frontmatter skill was silently accepted" "warning or error" "$output"
  fi
}

test_detect_invalid_bad_name_warns() {
  "$SKITTLE" init >/dev/null 2>&1
  # bad-name fixture: frontmatter name != directory name
  local output
  output=$("$SKITTLE" add "$FIXTURES_DIR/invalid/bad-name/SKILL.md" --name badname 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "warn|skip|mismatch|name"; then
    _pass "bad-name skill triggers warning or rejection"
  else
    _fail "bad-name skill was silently accepted" "warning or error" "$output"
  fi
}

test_detect_name_derived_from_directory() {
  "$SKITTLE" init >/dev/null 2>&1
  # Adding without --name should derive name from directory
  assert_exit_code 0 "$SKITTLE" add "$FIXTURES_DIR/flat-skills"
  assert_stdout_contains "flat-skills" "$SKITTLE" list
}

test_detect_plugin_json_metadata() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name meta-test >/dev/null 2>&1
  # list <skill> should reflect metadata from plugin.json
  local output
  output=$("$SKITTLE" list test-plugin/explore 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin"; then
    _pass "list shows plugin info from plugin.json"
  else
    _fail "plugin info not shown" "test-plugin in output" "$output"
  fi
}
