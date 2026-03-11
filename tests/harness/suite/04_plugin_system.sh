#!/usr/bin/env bash
# Suite 04: Plugin System
# Tests plugin list, plugin list --source, plugin show, implicit plugin naming.

test_plugin_list() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name src-b >/dev/null 2>&1
  # Should list all plugins across sources
  assert_exit_code 0 "$SKITTLE" plugin list
  assert_stdout_contains "test-plugin" "$SKITTLE" plugin list
  assert_stdout_contains "test-plugin-a" "$SKITTLE" plugin list
  assert_stdout_contains "test-plugin-b" "$SKITTLE" plugin list
}

test_plugin_list_filtered_by_source() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name src-b >/dev/null 2>&1
  # Filter to only src-b plugins
  local output
  output=$("$SKITTLE" plugin list --source src-b 2>/dev/null)
  assert_exit_code 0 "$SKITTLE" plugin list --source src-b
  if echo "$output" | grep -qF "test-plugin-a"; then
    _pass "filtered list contains src-b plugin"
  else
    _fail "filtered list missing src-b plugin" "test-plugin-a present" "$output"
  fi
  # Should not contain the plugin-source plugin
  if echo "$output" | grep -qF "test-plugin" && ! echo "$output" | grep -qF "test-plugin-a" && ! echo "$output" | grep -qF "test-plugin-b"; then
    _fail "filtered list contains wrong source plugin" "only src-b plugins" "$output"
  else
    _pass "filtered list excludes other sources"
  fi
}

test_plugin_show() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" plugin show test-plugin
  # Should show plugin name, version, and list of skills
  assert_stdout_contains "test-plugin" "$SKITTLE" plugin show test-plugin
  assert_stdout_contains "explore" "$SKITTLE" plugin show test-plugin
  assert_stdout_contains "apply" "$SKITTLE" plugin show test-plugin
  assert_stdout_contains "verify" "$SKITTLE" plugin show test-plugin
}

test_plugin_show_with_version() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  # plugin.toml declares version 0.1.0
  assert_stdout_contains "0.1.0" "$SKITTLE" plugin show test-plugin
}

test_plugin_show_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" plugin show nonexistent
}

test_implicit_plugin_from_flat_source() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/flat-skills" --name flat >/dev/null 2>&1
  # Flat dir gets an implicit plugin named after the source
  local output
  output=$("$SKITTLE" plugin list 2>/dev/null)
  assert_exit_code 0 "$SKITTLE" plugin list
  if echo "$output" | grep -qF "flat"; then
    _pass "implicit plugin created for flat source"
  else
    _fail "implicit plugin not found" "flat-related plugin name" "$output"
  fi
}

test_implicit_plugin_from_single_file() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/single-skill/SKILL.md" --name single >/dev/null 2>&1
  # Single file gets wrapped in implicit plugin
  local output
  output=$("$SKITTLE" plugin list 2>/dev/null)
  if echo "$output" | grep -qF "single"; then
    _pass "implicit plugin created for single file source"
  else
    _fail "implicit plugin not found for single file" "single-related plugin name" "$output"
  fi
}

test_plugin_skill_count() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name src >/dev/null 2>&1
  # test-plugin-a has 2 skills, test-plugin-b has 1
  local output
  output=$("$SKITTLE" plugin show test-plugin-a 2>/dev/null)
  assert_stdout_contains "skill-one" "$SKITTLE" plugin show test-plugin-a
  assert_stdout_contains "skill-two" "$SKITTLE" plugin show test-plugin-a
  output=$("$SKITTLE" plugin show test-plugin-b 2>/dev/null)
  assert_stdout_contains "skill-three" "$SKITTLE" plugin show test-plugin-b
}
