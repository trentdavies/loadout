#!/usr/bin/env bash
# Suite 05: Local Registry
# Tests registry.json creation, cache dir structure, skill identity resolution,
# ambiguous identity error.

test_registry_json_created_on_source_add() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_exists "$XDG_DATA_HOME/equip/.equip/registry.json"
}

test_registry_json_contains_source() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_contains "$XDG_DATA_HOME/equip/.equip/registry.json" "tp"
}

test_registry_json_contains_skills() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_file_contains "$XDG_DATA_HOME/equip/.equip/registry.json" "explore"
  assert_file_contains "$XDG_DATA_HOME/equip/.equip/registry.json" "apply"
  assert_file_contains "$XDG_DATA_HOME/equip/.equip/registry.json" "verify"
}

test_cache_dir_mirrors_source() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  assert_dir_exists "$XDG_DATA_HOME/equip/external/tp"
}

test_cache_contains_skill_files() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  # Cached source should contain the skill files
  local cache_dir="$XDG_DATA_HOME/equip/external/tp"
  # Look for SKILL.md somewhere in the cache
  local found
  found=$(find -L "$cache_dir" -name "SKILL.md" 2>/dev/null | head -1)
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
  # Short form with ambiguity: tool returns both matches (exit 0)
  local output
  output=$("$LOADOUT" list test-plugin/explore 2>&1)
  local exit_code=$?
  # Should succeed and show results from both sources
  if echo "$output" | grep -qiE "src-one|src-two|ambiguous"; then
    _pass "ambiguous identity shows both sources or disambiguation hint"
  else
    _fail "ambiguous identity missing source context" "src-one/src-two in output" "$output"
  fi
}

test_registry_cleared_on_source_remove() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  "$LOADOUT" remove tp --force >/dev/null 2>&1
  # Registry should no longer contain this source's entries
  if [ -f "$XDG_DATA_HOME/equip/.equip/registry.json" ]; then
    local content
    content=$(cat "$XDG_DATA_HOME/equip/.equip/registry.json")
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
  if [ -d "$custom_data/equip" ]; then
    _pass "custom XDG_DATA_HOME respected"
  else
    _fail "custom XDG_DATA_HOME not used" "$custom_data/equip exists" "not found"
  fi
  rm -rf "$custom_data"
}
