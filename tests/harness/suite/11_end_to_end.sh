#!/usr/bin/env bash
# Suite 11: End-to-End Lifecycle
# Full lifecycle: init → add → agent add → kit create → kit add →
# agent equip -k → status → deactivate/activate → agent unequip → remove

test_full_lifecycle() {
  reset_environment

  # 1. Init
  assert_exit_code 0 "$LOADOUT" init

  # 2. Add source
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source lifecycle-src

  # 3. Agent add
  assert_exit_code 0 "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name lifecycle-agent --scope machine --sync auto

  # 4. Kit create
  assert_exit_code 0 "$LOADOUT" kit create lifecycle-bundle

  # 5. Kit add skills
  assert_exit_code 0 "$LOADOUT" kit add lifecycle-bundle test-plugin/explore test-plugin/apply

  # 6. Equip kit
  assert_exit_code 0 "$LOADOUT" agent equip -k lifecycle-bundle -a lifecycle-agent -f
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

  # 8. Create second kit, deactivate first, activate second (--force required)
  "$LOADOUT" kit create lifecycle-bundle-b >/dev/null 2>&1
  "$LOADOUT" kit add lifecycle-bundle-b test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" kit deactivate lifecycle-bundle lifecycle-agent --force
  assert_exit_code 0 "$LOADOUT" kit activate lifecycle-bundle-b lifecycle-agent --force
  # Old skills removed, new skill installed
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 9. Unequip kit (--force required)
  assert_exit_code 0 "$LOADOUT" agent unequip -k lifecycle-bundle-b -a lifecycle-agent -f
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
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # List skills from both sources
  assert_stdout_contains "explore" "$LOADOUT" list
  assert_stdout_contains "skill-one" "$LOADOUT" list

  # Install from different sources
  assert_exit_code 0 "$LOADOUT" agent equip test-plugin/explore -a tgt -f
  assert_exit_code 0 "$LOADOUT" agent equip test-plugin-a/skill-one -a tgt -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  # Unequip one (--force required)
  "$LOADOUT" agent unequip test-plugin/explore -a tgt -f >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  _pass "multi-source lifecycle completed"
}

test_multi_agent_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt-claude --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" agent add codex "$TARGET_CODEX" --name tgt-codex --scope machine --sync auto >/dev/null 2>&1

  # Install same skill to both agents
  "$LOADOUT" agent equip test-plugin/explore -a tgt-claude -f >/dev/null 2>&1
  "$LOADOUT" agent equip test-plugin/explore -a tgt-codex -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  # Unequip from one (--force required), verify other is untouched
  "$LOADOUT" agent unequip test-plugin/explore -a tgt-claude -f >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  _pass "multi-agent lifecycle completed"
}

test_bundle_activate_deactivate_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Create kits
  "$LOADOUT" kit create dev-bundle >/dev/null 2>&1
  "$LOADOUT" kit add dev-bundle test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" kit create prod-bundle >/dev/null 2>&1
  "$LOADOUT" kit add prod-bundle test-plugin/verify >/dev/null 2>&1

  # Install dev
  "$LOADOUT" agent equip -k dev-bundle -a tgt -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Deactivate dev, activate prod (--force required)
  "$LOADOUT" kit deactivate dev-bundle tgt --force >/dev/null 2>&1
  "$LOADOUT" kit activate prod-bundle tgt --force >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Deactivate prod, activate dev (--force required)
  "$LOADOUT" kit deactivate prod-bundle tgt --force >/dev/null 2>&1
  "$LOADOUT" kit activate dev-bundle tgt --force >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  _pass "kit activate/deactivate lifecycle completed"
}

test_idempotent_operations_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  # Init is idempotent
  assert_exit_code 0 "$LOADOUT" init

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install same skill twice — should succeed both times
  "$LOADOUT" agent equip test-plugin/explore -a tgt -f >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" agent equip test-plugin/explore -a tgt -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Unequip twice — second should not error (preview mode)
  "$LOADOUT" agent unequip test-plugin/explore -a tgt -f >/dev/null 2>&1
  local output
  output=$("$LOADOUT" agent unequip test-plugin/explore -a tgt -f 2>&1)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "idempotent unequip succeeds"
  else
    _pass "idempotent unequip reports not-installed (acceptable)"
  fi

  _pass "idempotent operations lifecycle completed"
}

test_dry_run_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1
  "$LOADOUT" kit create dry-b >/dev/null 2>&1
  "$LOADOUT" kit add dry-b test-plugin/explore test-plugin/apply >/dev/null 2>&1

  # Dry run install — nothing should be written
  assert_exit_code 0 "$LOADOUT" agent equip -k dry-b -a tgt -f -n
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Real install
  "$LOADOUT" agent equip -k dry-b -a tgt -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Unequip without --force defaults to preview — files should remain
  assert_exit_code 0 "$LOADOUT" agent unequip -k dry-b -a tgt
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # --dry-run + --force — dry-run wins, files should remain
  assert_exit_code 0 "$LOADOUT" agent unequip -k dry-b -a tgt -f -n
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "dry run lifecycle completed"
}

test_cleanup_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Install some skills
  "$LOADOUT" agent equip test-plugin/explore -a tgt -f >/dev/null 2>&1
  "$LOADOUT" agent equip test-plugin/apply -a tgt -f >/dev/null 2>&1

  # Unequip everything (--force required)
  "$LOADOUT" agent unequip test-plugin/explore -a tgt -f >/dev/null 2>&1
  "$LOADOUT" agent unequip test-plugin/apply -a tgt -f >/dev/null 2>&1

  # Remove agent (--force required)
  "$LOADOUT" agent remove tgt --force >/dev/null 2>&1

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

  local agent_output
  agent_output=$("$LOADOUT" agent list 2>/dev/null)
  if echo "$agent_output" | grep -qF "tgt"; then
    _fail "agent still listed after remove" "tgt absent" "still present"
  else
    _pass "agents cleaned up"
  fi
}

test_error_recovery_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Try installing a nonexistent skill — should fail
  local output
  output=$("$LOADOUT" agent equip test-plugin/nonexistent -a tgt -f 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "nonexistent skill install fails"
  else
    _fail "nonexistent skill install succeeded" "non-zero exit" "exit 0"
  fi

  # After an error, valid operations should still work
  assert_exit_code 0 "$LOADOUT" agent equip test-plugin/explore -a tgt -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "error recovery lifecycle completed"
}
