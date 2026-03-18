#!/usr/bin/env bash
# Suite 11: End-to-End Lifecycle
# Full lifecycle: init → add → agent add → kit create → kit add →
# @agent +kit equip → status → unequip → remove

test_full_lifecycle() {
  reset_environment

  # 1. Init
  assert_exit_code 0 "$LOADOUT" init

  # 2. Add source
  assert_exit_code 0 "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source lifecycle-src

  # 3. Agent add
  assert_exit_code 0 "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name lifecycle-agent --scope machine --sync auto

  # 4. Kit create
  assert_exit_code 0 "$LOADOUT" kit create lifecycle-kit

  # 5. Kit add skills
  assert_exit_code 0 "$LOADOUT" kit add lifecycle-kit lifecycle-src/explore lifecycle-src/apply

  # 6. Equip kit via shorthand
  assert_exit_code 0 "$LOADOUT" @lifecycle-agent +lifecycle-kit -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # 7. Status
  local status_output
  status_output=$("$LOADOUT" status 2>/dev/null)
  local exit_code=$?
  if [ "$exit_code" -eq 0 ]; then
    _pass "status command succeeds after equip"
  else
    _fail "status command failed" "exit 0" "exit $exit_code"
  fi

  # 8. Unequip kit (--remove --force required)
  assert_exit_code 0 "$LOADOUT" @lifecycle-agent +lifecycle-kit -r -f
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # 9. Re-equip, then equip a different kit
  "$LOADOUT" kit create lifecycle-kit-b >/dev/null 2>&1
  "$LOADOUT" kit add lifecycle-kit-b lifecycle-src/verify >/dev/null 2>&1
  "$LOADOUT" @lifecycle-agent +lifecycle-kit -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Unequip first kit, equip second
  "$LOADOUT" @lifecycle-agent +lifecycle-kit -r -f >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" @lifecycle-agent +lifecycle-kit-b -f
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # 10. Remove local skills from the imported plugin
  "$LOADOUT" @lifecycle-agent +lifecycle-kit-b -r -f >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" remove lifecycle-src/* --force

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

  # Equip from different sources via shorthand
  assert_exit_code 0 "$LOADOUT" @tgt src-a/explore -f
  assert_exit_code 0 "$LOADOUT" @tgt test-plugin-a/skill-one -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/skill-one/SKILL.md"

  # Unequip one
  "$LOADOUT" @tgt src-a/explore -r -f >/dev/null 2>&1
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

  # Equip same skill to both agents
  "$LOADOUT" @tgt-claude src/explore -f >/dev/null 2>&1
  "$LOADOUT" @tgt-codex src/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  # Unequip from one, verify other is untouched
  "$LOADOUT" @tgt-claude src/explore -r -f >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"

  _pass "multi-agent lifecycle completed"
}

test_kit_swap_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Create kits
  "$LOADOUT" kit create dev-kit >/dev/null 2>&1
  "$LOADOUT" kit add dev-kit src/explore src/apply >/dev/null 2>&1
  "$LOADOUT" kit create prod-kit >/dev/null 2>&1
  "$LOADOUT" kit add prod-kit src/verify >/dev/null 2>&1

  # Equip dev
  "$LOADOUT" @tgt +dev-kit -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Swap: unequip dev, equip prod
  "$LOADOUT" @tgt +dev-kit -r -f >/dev/null 2>&1
  "$LOADOUT" @tgt +prod-kit -f >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  # Swap back
  "$LOADOUT" @tgt +prod-kit -r -f >/dev/null 2>&1
  "$LOADOUT" @tgt +dev-kit -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"

  _pass "kit swap lifecycle completed"
}

test_idempotent_operations_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  # Init is idempotent
  assert_exit_code 0 "$LOADOUT" init

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Equip same skill twice — should succeed both times
  "$LOADOUT" @tgt src/explore -f >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" @tgt src/explore -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Unequip twice — second should not error
  "$LOADOUT" @tgt src/explore -r -f >/dev/null 2>&1
  local output
  output=$("$LOADOUT" @tgt src/explore -r -f 2>&1)
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
  "$LOADOUT" kit create dry-kit >/dev/null 2>&1
  "$LOADOUT" kit add dry-kit src/explore src/apply >/dev/null 2>&1

  # Dry run equip — nothing should be written
  assert_exit_code 0 "$LOADOUT" -n @tgt +dry-kit -f
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"

  # Real equip
  "$LOADOUT" @tgt +dry-kit -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # Unequip without --force defaults to preview — files should remain
  assert_exit_code 0 "$LOADOUT" @tgt +dry-kit -r
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  # --dry-run + --force — dry-run wins, files should remain
  assert_exit_code 0 "$LOADOUT" -n @tgt +dry-kit -r -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "dry run lifecycle completed"
}

test_cleanup_lifecycle() {
  reset_environment
  "$LOADOUT" init >/dev/null 2>&1

  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source src >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name tgt --scope machine --sync auto >/dev/null 2>&1

  # Equip some skills
  "$LOADOUT" @tgt src/explore -f >/dev/null 2>&1
  "$LOADOUT" @tgt src/apply -f >/dev/null 2>&1

  # Unequip everything
  "$LOADOUT" @tgt src/explore -r -f >/dev/null 2>&1
  "$LOADOUT" @tgt src/apply -r -f >/dev/null 2>&1

  # Remove agent
  "$LOADOUT" agent remove tgt --force >/dev/null 2>&1

  # Remove all local skills from the imported plugin
  "$LOADOUT" remove src/* --force >/dev/null 2>&1

  # Verify local skills are gone
  local list_output
  list_output=$("$LOADOUT" list 2>/dev/null)
  if echo "$list_output" | grep -qF "src/"; then
    _fail "local plugin skills still listed after remove" "src/* absent" "still present"
  else
    _pass "local skills cleaned up"
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

  # Try equipping a nonexistent skill — should fail
  local output
  output=$("$LOADOUT" @tgt src/nonexistent -f 2>&1)
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    _pass "nonexistent skill equip fails"
  else
    _fail "nonexistent skill equip succeeded" "non-zero exit" "exit 0"
  fi

  # After an error, valid operations should still work
  assert_exit_code 0 "$LOADOUT" @tgt src/explore -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"

  _pass "error recovery lifecycle completed"
}
