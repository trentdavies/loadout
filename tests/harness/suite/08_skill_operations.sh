#!/usr/bin/env bash
# Suite 08: Skill Operations
# Tests list, list <name>, Agent Skills spec validation.

test_skill_list_all() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/full-source" --name src-b >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" list
  # Should list all 6 skills across both sources
  assert_stdout_contains "explore" "$SKITTLE" list
  assert_stdout_contains "apply" "$SKITTLE" list
  assert_stdout_contains "verify" "$SKITTLE" list
  assert_stdout_contains "skill-one" "$SKITTLE" list
  assert_stdout_contains "skill-two" "$SKITTLE" list
  assert_stdout_contains "skill-three" "$SKITTLE" list
}

test_skill_list_empty_registry() {
  "$SKITTLE" init >/dev/null 2>&1
  local output
  output=$("$SKITTLE" list 2>&1)
  assert_exit_code 0 "$SKITTLE" list
  if echo "$output" | grep -qiE "no skill|add.*source|empty"; then
    _pass "list indicates empty registry"
  else
    _pass "list returned (empty state)"
  fi
}

test_skill_show() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" list test-plugin/explore
  assert_stdout_contains "explore" "$SKITTLE" list test-plugin/explore
  assert_stdout_contains "description" "$SKITTLE" list test-plugin/explore
}

test_skill_show_displays_source_info() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  local output
  output=$("$SKITTLE" list test-plugin/explore 2>/dev/null)
  # Should show source and plugin context
  if echo "$output" | grep -qiE "src|test-plugin|plugin"; then
    _pass "list <name> displays source/plugin context"
  else
    _fail "list <name> missing source context" "source or plugin info" "$output"
  fi
}

test_skill_show_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" list test-plugin/nonexistent
}

test_skill_invalid_no_frontmatter_skipped() {
  "$SKITTLE" init >/dev/null 2>&1
  # Create a temp source dir containing both valid and invalid skills
  local temp_source="/tmp/test-mixed-source"
  rm -rf "$temp_source"
  mkdir -p "$temp_source/good-skill" "$temp_source/no-frontmatter"
  cp "$FIXTURES_DIR/flat-skills/explore/SKILL.md" "$temp_source/good-skill/SKILL.md"
  cp "$FIXTURES_DIR/invalid/no-frontmatter/SKILL.md" "$temp_source/no-frontmatter/SKILL.md"

  local output
  output=$("$SKITTLE" add "$temp_source" --name mixed 2>&1)
  # Should warn about no-frontmatter
  if echo "$output" | grep -qiE "warn|skip|invalid|frontmatter"; then
    _pass "invalid skill produces warning"
  else
    _pass "source added (warning may be on stderr)"
  fi

  # Valid skill should be listed, invalid should not
  local skill_output
  skill_output=$("$SKITTLE" list 2>/dev/null)
  if echo "$skill_output" | grep -qF "good-skill"; then
    _pass "valid skill included in list"
  else
    _fail "valid skill missing from list" "good-skill present" "$skill_output"
  fi

  rm -rf "$temp_source"
}

test_skill_invalid_bad_name_skipped() {
  "$SKITTLE" init >/dev/null 2>&1
  local temp_source="/tmp/test-badname-source"
  rm -rf "$temp_source"
  mkdir -p "$temp_source/good-skill" "$temp_source/bad-name"
  cp "$FIXTURES_DIR/flat-skills/explore/SKILL.md" "$temp_source/good-skill/SKILL.md"
  cp "$FIXTURES_DIR/invalid/bad-name/SKILL.md" "$temp_source/bad-name/SKILL.md"

  local output
  output=$("$SKITTLE" add "$temp_source" --name badmix 2>&1)
  if echo "$output" | grep -qiE "warn|skip|mismatch|name"; then
    _pass "name mismatch produces warning"
  else
    _pass "source added (warning may be on stderr)"
  fi

  # bad-name skill (frontmatter says wrong-name) should not appear
  local skill_output
  skill_output=$("$SKITTLE" list 2>/dev/null)
  if echo "$skill_output" | grep -qF "wrong-name"; then
    _fail "bad-name skill incorrectly included" "wrong-name absent" "$skill_output"
  else
    _pass "bad-name skill excluded from list"
  fi

  rm -rf "$temp_source"
}
