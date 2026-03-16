#!/usr/bin/env bash
# Suite 10: Kit Management
# Tests create, delete, list, show, add, drop, activate, deactivate.

test_bundle_create() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit create work-dev
  assert_stdout_contains "work-dev" "$LOADOUT" kit list
}

test_bundle_create_duplicate_errors() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" kit create dupe >/dev/null 2>&1
  local output
  output=$("$LOADOUT" kit create dupe 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate kit name rejected (exit $exit_code)"
  else
    _fail "duplicate kit name accepted" "non-zero exit" "exit 0"
  fi
}

test_bundle_delete() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" kit create to-delete >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit delete to-delete --force
  local output
  output=$("$LOADOUT" kit list 2>/dev/null)
  if echo "$output" | grep -qF "to-delete"; then
    _fail "kit still listed after delete" "to-delete absent" "still present"
  else
    _pass "kit removed from list"
  fi
}

test_bundle_delete_preview_default() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" kit create preview-b >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$LOADOUT" kit delete preview-b 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "kit delete defaults to preview mode"
  else
    _fail "kit delete did not show preview" "would/force message" "$output"
  fi
  # Kit should still exist
  assert_stdout_contains "preview-b" "$LOADOUT" kit list
}

test_bundle_delete_nonexistent() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 1 "$LOADOUT" kit delete nonexistent --force
}

test_bundle_list() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" kit create alpha >/dev/null 2>&1
  "$LOADOUT" kit create beta >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit list
  assert_stdout_contains "alpha" "$LOADOUT" kit list
  assert_stdout_contains "beta" "$LOADOUT" kit list
}

test_bundle_list_empty() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit list
}

test_bundle_show() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  "$LOADOUT" kit add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit show work
  assert_stdout_contains "explore" "$LOADOUT" kit show work
  assert_stdout_contains "apply" "$LOADOUT" kit show work
}

test_bundle_show_nonexistent() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 1 "$LOADOUT" kit show nonexistent
}

test_bundle_add_skills() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit add work test-plugin/explore
  assert_stdout_contains "explore" "$LOADOUT" kit show work
}

test_bundle_add_multiple_skills() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit add work test-plugin/explore test-plugin/apply test-plugin/verify
  assert_stdout_contains "explore" "$LOADOUT" kit show work
  assert_stdout_contains "apply" "$LOADOUT" kit show work
  assert_stdout_contains "verify" "$LOADOUT" kit show work
}

test_bundle_add_nonexistent_skill_errors() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  assert_exit_code 1 "$LOADOUT" kit add work test-plugin/nonexistent
}

test_bundle_add_duplicate_skill_informational() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  "$LOADOUT" kit add work test-plugin/explore >/dev/null 2>&1
  # Adding same skill again should not error
  local output
  output=$("$LOADOUT" kit add work test-plugin/explore 2>&1)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "duplicate skill add is not an error"
  else
    _fail "duplicate skill add errored" "exit 0" "exit $exit_code"
  fi
}

test_bundle_drop_skill() {
  setup_source_and_agents
  "$LOADOUT" kit create work >/dev/null 2>&1
  "$LOADOUT" kit add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit drop work test-plugin/explore
  local output
  output=$("$LOADOUT" kit show work 2>/dev/null)
  if echo "$output" | grep -qF "explore"; then
    _fail "dropped skill still in kit" "explore absent" "still present"
  else
    _pass "skill dropped from kit"
  fi
  # apply should still be there
  assert_stdout_contains "apply" "$LOADOUT" kit show work
}

test_bundle_deactivate_activate() {
  setup_source_and_agents
  # Create two kits
  "$LOADOUT" kit create bundle-a >/dev/null 2>&1
  "$LOADOUT" kit add bundle-a test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" kit create bundle-b >/dev/null 2>&1
  "$LOADOUT" kit add bundle-b test-plugin/verify >/dev/null 2>&1

  # Install bundle-a
  "$LOADOUT" agent equip -k bundle-a -a test-claude -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Deactivate bundle-a, then activate bundle-b (requires --force)
  assert_exit_code 0 "$LOADOUT" kit deactivate bundle-a test-claude --force
  assert_exit_code 0 "$LOADOUT" kit activate bundle-b test-claude --force
  # bundle-a skills should be gone
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  # bundle-b skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_bundle_activate_preview_default() {
  setup_source_and_agents
  "$LOADOUT" kit create ba >/dev/null 2>&1
  "$LOADOUT" kit add ba test-plugin/explore >/dev/null 2>&1

  # Without --force, activate should preview only
  local output
  output=$("$LOADOUT" kit activate ba test-claude 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "kit activate defaults to preview mode"
  else
    _fail "kit activate did not show preview" "would/force message" "$output"
  fi
  # explore should not be installed (no activate happened)
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}
