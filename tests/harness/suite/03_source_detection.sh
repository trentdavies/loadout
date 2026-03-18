#!/usr/bin/env bash
# Suite 03: Source Detection
# Tests progressive detection: single file, flat dir, plugin dir, full source,
# unrecognizable dir error, invalid skill warnings.

test_detect_single_file() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/single-skill/SKILL.md" --source single
  # Should result in one skill
  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "single-skill"; then
    _pass "single file detected as skill"
  else
    _fail "single file not detected" "single-skill in list" "$output"
  fi
}

test_detect_flat_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/flat-skills" --source flat
  # Should detect 2 skills: explore and apply
  assert_stdout_contains "explore" "$LOADOUT" list
  assert_stdout_contains "apply" "$LOADOUT" list
}

test_detect_repo_root_with_skills_subdir() {
  "$LOADOUT" init >/dev/null 2>&1

  local temp_source="/tmp/test-repo-root-skills"
  rm -rf "$temp_source"
  mkdir -p "$temp_source/skills/pptx/templates"
  cat >"$temp_source/README.md" <<'EOF'
# Example repo
EOF
  cat >"$temp_source/skills/pptx/SKILL.md" <<'EOF'
---
name: pptx
description: PowerPoint helper
---
EOF
  cat >"$temp_source/skills/pptx/templates/template.pptx" <<'EOF'
template
EOF

  assert_exit_code 0 "$LOADOUT" add "$temp_source" --source slides
  assert_stdout_contains "slides" "$LOADOUT" list
  assert_stdout_contains "pptx" "$LOADOUT" list
}

test_detect_plugin_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source plugged
  # Should detect plugin with 3 skills
  assert_stdout_contains "plugged" "$LOADOUT" list
  assert_stdout_contains "explore" "$LOADOUT" list
  assert_stdout_contains "apply" "$LOADOUT" list
  assert_stdout_contains "verify" "$LOADOUT" list
}

test_detect_full_source() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/full-source" --source full
  # Should detect 2 plugins (visible in list table columns)
  assert_stdout_contains "test-plugin-a" "$LOADOUT" list
  assert_stdout_contains "test-plugin-b" "$LOADOUT" list
  # Should detect 3 skills total
  assert_stdout_contains "skill-one" "$LOADOUT" list
  assert_stdout_contains "skill-two" "$LOADOUT" list
  assert_stdout_contains "skill-three" "$LOADOUT" list
}

test_detect_unrecognizable_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  # empty-dir has no SKILL.md, no toml — should fail
  assert_exit_code 1 "$LOADOUT" add "$FIXTURES_DIR/invalid/empty-dir" --source bad
  assert_stderr_contains "error" "$LOADOUT" add "$FIXTURES_DIR/invalid/empty-dir" --source bad
}

test_detect_invalid_no_frontmatter_warns() {
  "$LOADOUT" init >/dev/null 2>&1
  # Adding a source that contains no-frontmatter skill should warn and skip it
  local output
  output=$("$LOADOUT" add "$FIXTURES_DIR/invalid/no-frontmatter/SKILL.md" --source nofm 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "warn|skip|invalid|frontmatter"; then
    _pass "no-frontmatter skill triggers warning or rejection"
  else
    _fail "no-frontmatter skill was silently accepted" "warning or error" "$output"
  fi
}

test_detect_invalid_bad_name_warns() {
  "$LOADOUT" init >/dev/null 2>&1
  # bad-name fixture: frontmatter name != directory name
  local output
  output=$("$LOADOUT" add "$FIXTURES_DIR/invalid/bad-name/SKILL.md" --source badname 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "warn|skip|mismatch|name"; then
    _pass "bad-name skill triggers warning or rejection"
  else
    _fail "bad-name skill was silently accepted" "warning or error" "$output"
  fi
}

test_detect_name_derived_from_directory() {
  "$LOADOUT" init >/dev/null 2>&1
  # Adding without --name should derive name from directory
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/flat-skills"
  assert_stdout_contains "flat-skills" "$LOADOUT" list
}

test_detect_plugin_json_metadata() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" >/dev/null 2>&1
  # list <skill> should reflect metadata from plugin.json
  local output
  output=$("$LOADOUT" list test-plugin/explore 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin"; then
    _pass "list shows plugin info from plugin.json"
  else
    _fail "plugin info not shown" "test-plugin in output" "$output"
  fi
}
