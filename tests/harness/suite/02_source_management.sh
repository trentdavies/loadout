#!/usr/bin/env bash
# Suite 02: Source Management
# Tests add (local + git), remove, update, duplicate name error.

test_source_add_local() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source
  assert_stdout_contains "test-source" "$LOADOUT" source list
}

test_source_add_local_full_source() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source test-plugin
  assert_stdout_contains "test-plugin" "$LOADOUT" list
}

test_source_add_local_plugin_path_creates_external_source() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source add "$FIXTURES_DIR/plugin-source" --source plugin-src
  assert_stdout_contains "plugin-src" "$LOADOUT" source list
  assert_stdout_contains "plugin-src:test-plugin/explore" "$LOADOUT" list

  local cache_path="$XDG_DATA_HOME/equip/external/plugin-src"
  if [ -L "$cache_path" ]; then
    _pass "local plugin path added as symlinked external source"
  else
    _fail "local plugin path was not symlinked into external sources" "symlink at $cache_path" "missing or not symlink"
  fi

  if [ -d "$XDG_DATA_HOME/equip/test-plugin" ]; then
    _fail "local plugin path was incorrectly imported into local source" "no local plugin dir" "found $XDG_DATA_HOME/equip/test-plugin"
  else
    _pass "local plugin path not imported into local source"
  fi
}

test_source_add_local_flat_skills_honors_plugin_override() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source add "$FIXTURES_DIR/flat-skills" --source flat-src --plugin curated
  assert_stdout_contains "flat-src:curated/explore" "$LOADOUT" list
  assert_stdout_contains "flat-src:curated/apply" "$LOADOUT" list
}

test_source_add_local_single_skill_honors_skill_override() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source add "$FIXTURES_DIR/single-skill/SKILL.md" --source single-src --plugin custom-plugin --skill renamed-skill
  assert_stdout_contains "single-src:custom-plugin/renamed-skill" "$LOADOUT" list
}

test_source_add_git() {
  skip_if_no_network && return
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source add https://github.com/anthropics/skills.git --source anthropic-skills
  assert_stdout_contains "anthropic-skills" "$LOADOUT" source list
}

test_source_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source remove test-source --force
  # Should no longer appear in list
  local output
  output=$("$LOADOUT" source list 2>/dev/null)
  if echo "$output" | grep -qF "test-source"; then
    _fail "source still listed after remove" "test-source absent" "still present"
  else
    _pass "source removed from list"
  fi
}

test_source_remove_preview_default() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$LOADOUT" source remove test-source 2>&1)
  if echo "$output" | grep -qiE "would|force|confirm"; then
    _pass "remove defaults to preview mode"
  else
    _fail "remove did not show preview" "would/force message" "$output"
  fi
  # Source should still be listed
  assert_stdout_contains "test-source" "$LOADOUT" source list
}

test_source_update() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source update test-source
}

test_source_update_all() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" source update
}

test_source_duplicate_name_error() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source dupe >/dev/null 2>&1
  # Adding with the same name again should error
  local output
  output=$("$LOADOUT" source add "$FIXTURES_DIR/full-source" --source dupe 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate source name rejected (exit $exit_code)"
  else
    _fail "duplicate source name was accepted" "non-zero exit" "exit 0"
  fi
}

test_source_remove_with_installed_skills_warns() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" @test-claude test-plugin-a/skill-one -f >/dev/null 2>&1
  # Without --force, should preview and warn about installed skills
  local output
  output=$("$LOADOUT" source remove test-source 2>&1)
  if echo "$output" | grep -qiE "warn|force|installed|would"; then
    _pass "remove warns about installed skills"
  else
    _fail "remove did not warn about installed skills" "warning or preview" "$output"
  fi
  # Source should still exist (no --force)
  assert_stdout_contains "test-source" "$LOADOUT" source list
}

test_top_level_remove_local_skill() {
  "$LOADOUT" init >/dev/null 2>&1
  cp -R "$FIXTURES_DIR/plugin-source" "$XDG_DATA_HOME/equip/test-plugin"
  "$LOADOUT" reconcile --source local >/dev/null 2>&1

  assert_exit_code 0 "$LOADOUT" remove test-plugin/explore --force

  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "test-plugin/explore"; then
    _fail "local skill still listed after remove" "test-plugin/explore absent" "still present"
  else
    _pass "top-level remove deleted local skill"
  fi

  if echo "$output" | grep -qF "test-plugin/apply"; then
    _pass "other local skills preserved"
  else
    _fail "unexpected extra local skill removal" "test-plugin/apply present" "$output"
  fi
}

test_top_level_remove_source_fallback() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" source add "$FIXTURES_DIR/full-source" --source test-source >/dev/null 2>&1

  assert_exit_code 0 "$LOADOUT" remove test-source --force
  local output
  output=$("$LOADOUT" source list 2>/dev/null)
  if echo "$output" | grep -qF "test-source"; then
    _fail "source still listed after top-level fallback remove" "test-source absent" "still present"
  else
    _pass "top-level remove falls back to source removal"
  fi
}
