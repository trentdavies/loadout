#!/usr/bin/env bash
# Suite 05: Local Registry
# Tests registry.json creation, cache dir structure, skill identity resolution,
# ambiguous identity error.

test_registry_json_created_on_source_add() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_exists "$XDG_DATA_HOME/loadout/.loadout/registry.json"
}

test_registry_json_contains_source() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_contains "$XDG_DATA_HOME/loadout/.loadout/registry.json" "tp"
}

test_registry_json_contains_skills() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_contains "$XDG_DATA_HOME/loadout/.loadout/registry.json" "explore"
  assert_file_contains "$XDG_DATA_HOME/loadout/.loadout/registry.json" "apply"
  assert_file_contains "$XDG_DATA_HOME/loadout/.loadout/registry.json" "verify"
}

test_cache_dir_mirrors_source() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_dir_exists "$XDG_DATA_HOME/loadout/external/tp"
}

test_cache_contains_skill_files() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  # Cached source should contain the skill files
  local cache_dir="$XDG_DATA_HOME/loadout/external/tp"
  # Look for SKILL.md somewhere in the cache
  local found
  found=$(find "$cache_dir" -name "SKILL.md" 2>/dev/null | head -1)
  if [ -n "$found" ]; then
    _pass "cache contains SKILL.md files"
  else
    _fail "cache missing SKILL.md files" "SKILL.md in $cache_dir" "not found"
  fi
}

test_skill_identity_short_form() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  # Should be able to look up by plugin/skill
  assert_exit_code 0 "$LOADOUT" list test-plugin/explore
}

test_skill_identity_full_form() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  # Full form: source:plugin/skill
  assert_exit_code 0 "$LOADOUT" list tp:test-plugin/explore
}

test_skill_identity_ambiguous_error() {
  "$LOADOUT" init >/dev/null 2>&1
  # Add the same plugin name from two different sources to create ambiguity
  # We'll use plugin-source twice with different source names
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src-one >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src-two >/dev/null 2>&1
  # Short form should be ambiguous now
  local output
  output=$("$LOADOUT" list test-plugin/explore 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "ambiguous skill identity returns error (exit $exit_code)"
  else
    _fail "ambiguous skill identity did not error" "non-zero exit" "exit 0"
  fi
  # Error message should mention both sources or disambiguation
  if echo "$output" | grep -qiE "ambiguous|multiple|disambiguate|src-one|src-two"; then
    _pass "ambiguity error mentions conflicting sources"
  else
    _fail "ambiguity error lacks context" "source names or disambiguation hint" "$output"
  fi
}

test_registry_cleared_on_source_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  "$LOADOUT" remove tp --force >/dev/null 2>&1
  # Registry should no longer contain this source's entries
  if [ -f "$XDG_DATA_HOME/loadout/.loadout/registry.json" ]; then
    local content
    content=$(cat "$XDG_DATA_HOME/loadout/.loadout/registry.json")
    if echo "$content" | grep -qF "tp"; then
      _fail "registry still contains removed source" "tp absent" "still present"
    else
      _pass "registry cleared after source remove"
    fi
  else
    _pass "registry file removed (clean state)"
  fi
}

test_xdg_override_respected() {
  # Use a custom XDG path
  local custom_data="/tmp/test-custom-xdg"
  rm -rf "$custom_data"
  XDG_DATA_HOME="$custom_data" "$LOADOUT" init >/dev/null 2>&1
  XDG_DATA_HOME="$custom_data" "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  if [ -d "$custom_data/loadout" ]; then
    _pass "custom XDG_DATA_HOME respected"
  else
    _fail "custom XDG_DATA_HOME not used" "$custom_data/loadout exists" "not found"
  fi
  rm -rf "$custom_data"
}
