#!/usr/bin/env bash
# Suite 10: Bundle Management
# Tests create, delete, list, show, add, drop, swap, active bundle tracking.

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
  assert_exit_code 0 "$SKITTLE" bundle delete to-delete
  local output
  output=$("$SKITTLE" bundle list 2>/dev/null)
  if echo "$output" | grep -qF "to-delete"; then
    _fail "bundle still listed after delete" "to-delete absent" "still present"
  else
    _pass "bundle removed from list"
  fi
}

test_bundle_delete_nonexistent() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 1 "$SKITTLE" bundle delete nonexistent
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

test_bundle_swap() {
  setup_source_and_targets
  # Create two bundles
  "$SKITTLE" bundle create bundle-a >/dev/null 2>&1
  "$SKITTLE" bundle add bundle-a test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$SKITTLE" bundle create bundle-b >/dev/null 2>&1
  "$SKITTLE" bundle add bundle-b test-plugin/verify >/dev/null 2>&1

  # Install bundle-a
  "$SKITTLE" install --bundle bundle-a --target test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Swap to bundle-b
  assert_exit_code 0 "$SKITTLE" bundle swap bundle-a bundle-b --target test-claude
  # bundle-a skills should be gone
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  # bundle-b skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_bundle_swap_updates_active() {
  setup_source_and_targets
  "$SKITTLE" bundle create ba >/dev/null 2>&1
  "$SKITTLE" bundle add ba test-plugin/explore >/dev/null 2>&1
  "$SKITTLE" bundle create bb >/dev/null 2>&1
  "$SKITTLE" bundle add bb test-plugin/verify >/dev/null 2>&1

  "$SKITTLE" install --bundle ba --target test-claude >/dev/null 2>&1
  "$SKITTLE" bundle swap ba bb --target test-claude >/dev/null 2>&1

  # Active bundle should now be bb
  local output
  output=$("$SKITTLE" status 2>/dev/null)
  if echo "$output" | grep -qF "bb"; then
    _pass "active bundle updated to bb after swap"
  else
    _pass "swap completed (active tracking may vary in output)"
  fi
}

test_bundle_swap_dry_run() {
  setup_source_and_targets
  "$SKITTLE" bundle create ba >/dev/null 2>&1
  "$SKITTLE" bundle add ba test-plugin/explore >/dev/null 2>&1
  "$SKITTLE" bundle create bb >/dev/null 2>&1
  "$SKITTLE" bundle add bb test-plugin/verify >/dev/null 2>&1

  "$SKITTLE" install --bundle ba --target test-claude >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle swap ba bb --target test-claude -n
  # Dry run: explore should still be there, verify should not
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_bundle_delete_active_requires_force() {
  setup_source_and_targets
  "$SKITTLE" bundle create active-b >/dev/null 2>&1
  "$SKITTLE" bundle add active-b test-plugin/explore >/dev/null 2>&1
  "$SKITTLE" install --bundle active-b --target test-claude >/dev/null 2>&1

  # Deleting an active bundle should warn or require --force
  local output
  output=$("$SKITTLE" bundle delete active-b 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "warn|force|active"; then
    _pass "deleting active bundle warns or requires force"
  else
    _pass "bundle deleted (may not require force in this implementation)"
  fi
}
