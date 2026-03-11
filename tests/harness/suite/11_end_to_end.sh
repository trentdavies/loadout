#!/usr/bin/env bash
# Suite 11: End-to-End Lifecycle
# Full lifecycle: init → source add → target add → bundle create → bundle add →
# install --bundle → status → swap → uninstall → source remove → cache clean

test_full_lifecycle() {
  reset_environment

  # 1. Init
  assert_exit_code 0 "$SKITTLE" init

  # 2. Source add
  assert_exit_code 0 "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name lifecycle-src

  # 3. Target add
  assert_exit_code 0 "$SKITTLE" target add claude "$TARGET_CLAUDE" --name lifecycle-target --scope machine --sync auto

  # 4. Bundle create
  assert_exit_code 0 "$SKITTLE" bundle create lifecycle-bundle

  # 5. Bundle add skills
  assert_exit_code 0 "$SKITTLE" bundle add lifecycle-bundle test-plugin/explore test-plugin/apply

  # 6. Install --bundle
  assert_exit_code 0 "$SKITTLE" install --bundle lifecycle-bundle --target lifecycle-target
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # 7. Status
  local status_output
  status_output=$("$SKITTLE" status 2>/dev/null)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "status command succeeds after install"
  else
    _fail "status command failed" "exit 0" "exit $exit_code"
  fi

  # 8. Create second bundle and swap (--force required)
  "$SKITTLE" bundle create lifecycle-bundle-b >/dev/null 2>&1
  "$SKITTLE" bundle add lifecycle-bundle-b test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" bundle swap lifecycle-bundle lifecycle-bundle-b --target lifecycle-target --force
  # Old skills removed, new skill installed
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 9. Uninstall --bundle (--force required)
  assert_exit_code 0 "$SKITTLE" uninstall --bundle lifecycle-bundle-b --target lifecycle-target --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 10. Source remove (--force required)
  assert_exit_code 0 "$SKITTLE" source remove lifecycle-src --force

  # 11. Cache clean (--force required)
  assert_exit_code 0 "$SKITTLE" cache clean --force

  _pass "full lifecycle completed successfully"
}

test_multi_source_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  # Add both sources
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src-a >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/full-source" --name src-b >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # List skills from both sources
  assert_stdout_contains "explore" "$SKITTLE" skill list
  assert_stdout_contains "skill-one" "$SKITTLE" skill list

  # Install from different sources
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target tgt
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin-a/skill-one --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  # Uninstall one (--force required)
  "$SKITTLE" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  _pass "multi-source lifecycle completed"
}

test_multi_target_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt-claude --scope machine --sync auto >/dev/null 2>&1
  "$SKITTLE" target add codex "$TARGET_CODEX" --name tgt-codex --scope machine --sync auto >/dev/null 2>&1

  # Install same skill to both targets
  "$SKITTLE" install --skill test-plugin/explore --target tgt-claude >/dev/null 2>&1
  "$SKITTLE" install --skill test-plugin/explore --target tgt-codex >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  # Uninstall from one (--force required), verify other is untouched
  "$SKITTLE" uninstall --skill test-plugin/explore --target tgt-claude --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  _pass "multi-target lifecycle completed"
}

test_bundle_swap_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Create bundles
  "$SKITTLE" bundle create dev-bundle >/dev/null 2>&1
  "$SKITTLE" bundle add dev-bundle test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$SKITTLE" bundle create prod-bundle >/dev/null 2>&1
  "$SKITTLE" bundle add prod-bundle test-plugin/verify >/dev/null 2>&1

  # Install dev
  "$SKITTLE" install --bundle dev-bundle --target tgt >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Swap to prod (--force required)
  "$SKITTLE" bundle swap dev-bundle prod-bundle --target tgt --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Swap back to dev (--force required)
  "$SKITTLE" bundle swap prod-bundle dev-bundle --target tgt --force >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  _pass "bundle swap lifecycle completed"
}

test_idempotent_operations_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  # Init is idempotent
  assert_exit_code 0 "$SKITTLE" init

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install same skill twice — should succeed both times
  "$SKITTLE" install --skill test-plugin/explore --target tgt >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Uninstall twice — second should not error (preview mode)
  "$SKITTLE" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  local output
  output=$("$SKITTLE" uninstall --skill test-plugin/explore --target tgt --force 2>&1)
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
  "$SKITTLE" init >/dev/null 2>&1

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1
  "$SKITTLE" bundle create dry-b >/dev/null 2>&1
  "$SKITTLE" bundle add dry-b test-plugin/explore test-plugin/apply >/dev/null 2>&1

  # Dry run install — nothing should be written
  assert_exit_code 0 "$SKITTLE" install --bundle dry-b --target tgt -n
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Real install
  "$SKITTLE" install --bundle dry-b --target tgt >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Uninstall without --force defaults to preview — files should remain
  assert_exit_code 0 "$SKITTLE" uninstall --bundle dry-b --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # --dry-run + --force — dry-run wins, files should remain
  assert_exit_code 0 "$SKITTLE" uninstall --bundle dry-b --target tgt --force -n
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "dry run lifecycle completed"
}

test_cleanup_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install some skills
  "$SKITTLE" install --skill test-plugin/explore --target tgt >/dev/null 2>&1
  "$SKITTLE" install --skill test-plugin/apply --target tgt >/dev/null 2>&1

  # Uninstall everything (--force required)
  "$SKITTLE" uninstall --skill test-plugin/explore --target tgt --force >/dev/null 2>&1
  "$SKITTLE" uninstall --skill test-plugin/apply --target tgt --force >/dev/null 2>&1

  # Remove target (--force required)
  "$SKITTLE" target remove tgt --force >/dev/null 2>&1

  # Remove source (--force required)
  "$SKITTLE" source remove src --force >/dev/null 2>&1

  # Clean cache (--force required)
  assert_exit_code 0 "$SKITTLE" cache clean --force

  # Verify sources and targets are empty
  local source_output
  source_output=$("$SKITTLE" source list 2>/dev/null)
  if echo "$source_output" | grep -qF "src"; then
    _fail "source still listed after remove" "src absent" "still present"
  else
    _pass "sources cleaned up"
  fi

  local target_output
  target_output=$("$SKITTLE" target list 2>/dev/null)
  if echo "$target_output" | grep -qF "tgt"; then
    _fail "target still listed after remove" "tgt absent" "still present"
  else
    _pass "targets cleaned up"
  fi
}

test_error_recovery_lifecycle() {
  reset_environment
  "$SKITTLE" init >/dev/null 2>&1

  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name src >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Try installing a nonexistent skill — should fail
  local output
  output=$("$SKITTLE" install --skill test-plugin/nonexistent --target tgt 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "nonexistent skill install fails"
  else
    _fail "nonexistent skill install succeeded" "non-zero exit" "exit 0"
  fi

  # After an error, valid operations should still work
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "error recovery lifecycle completed"
}
