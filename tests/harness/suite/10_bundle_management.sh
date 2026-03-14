#!/usr/bin/env bash
# Suite 10: Bundle Management
# Tests create, delete, list, show, add, drop, activate, deactivate.

test_bundle_create() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle create work-dev
  assert_stdout_contains "work-dev" "$SKITTLE" bundle list
}

test_bundle_create_duplicate_errors() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" bundle create dupe >/dev/null 2>&1
  local output
  output=$("$SKITTLE" bundle create dupe 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "duplicate bundle name rejected (exit $exit_code)"
  else
    _fail "duplicate bundle name accepted" "non-zero exit" "exit 0"
  fi
}

test_bundle_delete() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" bundle create to-delete >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle delete to-delete --force
  local output
  output=$("$SKITTLE" bundle list 2>/dev/null)
  if echo "$output" | grep -qF "to-delete"; then
    _fail "bundle still listed after delete" "to-delete absent" "still present"
  else
    _pass "bundle removed from list"
  fi
}

test_bundle_delete_preview_default() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" bundle create preview-b >/dev/null 2>&1
  # Without --force, should preview
  local output
  output=$("$SKITTLE" bundle delete preview-b 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "bundle delete defaults to preview mode"
  else
    _fail "bundle delete did not show preview" "would/force message" "$output"
  fi
  # Bundle should still exist
  assert_stdout_contains "preview-b" "$SKITTLE" bundle list
}

test_bundle_delete_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" bundle delete nonexistent --force
}

test_bundle_list() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" bundle create alpha >/dev/null 2>&1
  "$SKITTLE" bundle create beta >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle list
  assert_stdout_contains "alpha" "$SKITTLE" bundle list
  assert_stdout_contains "beta" "$SKITTLE" bundle list
}

test_bundle_list_empty() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle list
}

test_bundle_show() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  "$SKITTLE" bundle add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle show work
  assert_stdout_contains "explore" "$SKITTLE" bundle show work
  assert_stdout_contains "apply" "$SKITTLE" bundle show work
}

test_bundle_show_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" bundle show nonexistent
}

test_bundle_add_skills() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle add work test-plugin/explore
  assert_stdout_contains "explore" "$SKITTLE" bundle show work
}

test_bundle_add_multiple_skills() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle add work test-plugin/explore test-plugin/apply test-plugin/verify
  assert_stdout_contains "explore" "$SKITTLE" bundle show work
  assert_stdout_contains "apply" "$SKITTLE" bundle show work
  assert_stdout_contains "verify" "$SKITTLE" bundle show work
}

test_bundle_add_nonexistent_skill_errors() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" bundle add work test-plugin/nonexistent
}

test_bundle_add_duplicate_skill_informational() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  "$SKITTLE" bundle add work test-plugin/explore >/dev/null 2>&1
  # Adding same skill again should not error
  local output
  output=$("$SKITTLE" bundle add work test-plugin/explore 2>&1)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "duplicate skill add is not an error"
  else
    _fail "duplicate skill add errored" "exit 0" "exit $exit_code"
  fi
}

test_bundle_drop_skill() {
  setup_source_and_targets
  "$SKITTLE" bundle create work >/dev/null 2>&1
  "$SKITTLE" bundle add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle drop work test-plugin/explore
  local output
  output=$("$SKITTLE" bundle show work 2>/dev/null)
  if echo "$output" | grep -qF "explore"; then
    _fail "dropped skill still in bundle" "explore absent" "still present"
  else
    _pass "skill dropped from bundle"
  fi
  # apply should still be there
  assert_stdout_contains "apply" "$SKITTLE" bundle show work
}

test_bundle_deactivate_activate() {
  setup_source_and_targets
  # Create two bundles
  "$SKITTLE" bundle create bundle-a >/dev/null 2>&1
  "$SKITTLE" bundle add bundle-a test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$SKITTLE" bundle create bundle-b >/dev/null 2>&1
  "$SKITTLE" bundle add bundle-b test-plugin/verify >/dev/null 2>&1

  # Install bundle-a
  "$SKITTLE" apply --force --bundle bundle-a --target test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Deactivate bundle-a, then activate bundle-b (requires --force)
  assert_exit_code 0 "$SKITTLE" bundle deactivate bundle-a test-claude --force
  assert_exit_code 0 "$SKITTLE" bundle activate bundle-b test-claude --force
  # bundle-a skills should be gone
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  # bundle-b skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_bundle_activate_preview_default() {
  setup_source_and_targets
  "$SKITTLE" bundle create ba >/dev/null 2>&1
  "$SKITTLE" bundle add ba test-plugin/explore >/dev/null 2>&1

  # Without --force, activate should preview only
  local output
  output=$("$SKITTLE" bundle activate ba test-claude 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "bundle activate defaults to preview mode"
  else
    _fail "bundle activate did not show preview" "would/force message" "$output"
  fi
  # explore should not be installed (no activate happened)
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}
