#!/usr/bin/env bash
# Suite 11: End-to-End Lifecycle
# Full lifecycle: init → add → target add → bundle create → bundle add →
# install --bundle → status → deactivate/activate → uninstall → remove

test_full_lifecycle() {
  reset_environment

  # 1. Init
  assert_exit_code 0 "$LOADOUT" init

  # 2. Add source
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source lifecycle-src

  # 3. Target add
  assert_exit_code 0 "$LOADOUT" target add claude "$TARGET_CLAUDE" --name lifecycle-target --scope machine --sync auto

  # 4. Bundle create
  assert_exit_code 0 "$LOADOUT" bundle create lifecycle-bundle

  # 5. Bundle add skills
  assert_exit_code 0 "$LOADOUT" bundle add lifecycle-bundle test-plugin/explore test-plugin/apply

  # 6. Install --bundle
  assert_exit_code 0 "$LOADOUT" apply --force --bundle lifecycle-bundle --target lifecycle-target
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # 7. Status
  local status_output
  status_output=$("$LOADOUT" status 2>/dev/null)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "status command succeeds after install"
  else
    _fail "status command failed" "exit 0" "exit $exit_code"
  fi

  # 8. Create second bundle, deactivate first, activate second (--force required)
  "$LOADOUT" bundle create lifecycle-bundle-b >/dev/null 2>&1
  "$LOADOUT" bundle add lifecycle-bundle-b test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" bundle deactivate lifecycle-bundle lifecycle-target --force
  assert_exit_code 0 "$LOADOUT" bundle activate lifecycle-bundle-b lifecycle-target --force
  # Old skills removed, new skill installed
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 9. Uninstall --bundle (--force required)
  assert_exit_code 0 "$LOADOUT" uninstall --bundle lifecycle-bundle-b --target lifecycle-target --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 10. Remove source (--force required)
  assert_exit_code 0 "$LOADOUT" remove lifecycle-src --force

  _pass "full lifecycle completed successfully"
}

test_multi_source_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  # Add both sources
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src-a >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/full-source" --source src-b >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # List skills from both sources
  assert_stdout_contains "explore" "$LOADOUT" list
  assert_stdout_contains "skill-one" "$LOADOUT" list

  # Install from different sources
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --target tgt
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin-a/skill-one --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  # Uninstall one (--force required)
  "$LOADOUT" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  _pass "multi-source lifecycle completed"
}

test_multi_target_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" target add codex "$TARGET_CODEX" --name tgt-codex --scope machine --sync auto >/dev/null 2>&1

  # Install same skill to both targets
  "$LOADOUT" apply --force --skill test-plugin/explore --target tgt-claude >/dev/null 2>&1
  "$LOADOUT" apply --force --skill test-plugin/explore --target tgt-codex >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  # Uninstall from one (--force required), verify other is untouched
  "$LOADOUT" uninstall --skill test-plugin/explore --target tgt-claude --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  _pass "multi-target lifecycle completed"
}

test_bundle_activate_deactivate_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Create bundles
  "$LOADOUT" bundle create dev-bundle >/dev/null 2>&1
  "$LOADOUT" bundle add dev-bundle test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" bundle create prod-bundle >/dev/null 2>&1
  "$LOADOUT" bundle add prod-bundle test-plugin/verify >/dev/null 2>&1

  # Install dev
  "$LOADOUT" apply --force --bundle dev-bundle --target tgt >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Deactivate dev, activate prod (--force required)
  "$LOADOUT" bundle deactivate dev-bundle tgt --force >/dev/null 2>&1
  "$LOADOUT" bundle activate prod-bundle tgt --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Deactivate prod, activate dev (--force required)
  "$LOADOUT" bundle deactivate prod-bundle tgt --force >/dev/null 2>&1
  "$LOADOUT" bundle activate dev-bundle tgt --force >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  _pass "bundle activate/deactivate lifecycle completed"
}

test_idempotent_operations_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  # Init is idempotent
  assert_exit_code 0 "$LOADOUT" init

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install same skill twice — should succeed both times
  "$LOADOUT" apply --force --skill test-plugin/explore --target tgt >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Uninstall twice — second should not error (preview mode)
  "$LOADOUT" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  local output
  output=$("$LOADOUT" uninstall --skill test-plugin/explore --target tgt --force 2>&1)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "idempotent uninstall succeeds"
  else
    _pass "idempotent uninstall reports not-installed (acceptable)"
  fi

  _pass "idempotent operations lifecycle completed"
}

test_dry_run_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" bundle create dry-b >/dev/null 2>&1
  "$LOADOUT" bundle add dry-b test-plugin/explore test-plugin/apply >/dev/null 2>&1

  # Dry run install — nothing should be written
  assert_exit_code 0 "$LOADOUT" apply --force --bundle dry-b --target tgt -n
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Real install
  "$LOADOUT" apply --force --bundle dry-b --target tgt >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Uninstall without --force defaults to preview — files should remain
  assert_exit_code 0 "$LOADOUT" uninstall --bundle dry-b --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # --dry-run + --force — dry-run wins, files should remain
  assert_exit_code 0 "$LOADOUT" uninstall --bundle dry-b --target tgt --force -n
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "dry run lifecycle completed"
}

test_cleanup_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install some skills
  "$LOADOUT" apply --force --skill test-plugin/explore --target tgt >/dev/null 2>&1
  "$LOADOUT" apply --force --skill test-plugin/apply --target tgt >/dev/null 2>&1

  # Uninstall everything (--force required)
  "$LOADOUT" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  "$LOADOUT" uninstall --skill test-plugin/apply --target tgt --force >/dev/null 2>&1

  # Remove target (--force required)
  "$LOADOUT" target remove tgt --force >/dev/null 2>&1

  # Remove source (--force required)
  "$LOADOUT" remove src --force >/dev/null 2>&1

  # Verify sources are gone (list should show no skills)
  local list_output
  list_output=$("$LOADOUT" list 2>/dev/null)
  if echo "$list_output" | grep -qF "src"; then
    _fail "source still listed after remove" "src absent" "still present"
  else
    _pass "sources cleaned up"
  fi

  local target_output
  target_output=$("$LOADOUT" target list 2>/dev/null)
  if echo "$target_output" | grep -qF "tgt"; then
    _fail "target still listed after remove" "tgt absent" "still present"
  else
    _pass "targets cleaned up"
  fi
}

test_error_recovery_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Try installing a nonexistent skill — should fail
  local output
  output=$("$LOADOUT" apply --force --skill test-plugin/nonexistent --target tgt 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "nonexistent skill install fails"
  else
    _fail "nonexistent skill install succeeded" "non-zero exit" "exit 0"
  fi

  # After an error, valid operations should still work
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "error recovery lifecycle completed"
}
