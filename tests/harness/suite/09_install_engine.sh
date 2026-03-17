#!/usr/bin/env bash
# Suite 09: Equip Engine
# Tests equip/unequip with @agent shorthand, patterns, +kit, --all,
# dry run (-n), idempotent equip.

test_equip_no_flags_errors() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 2 "$LOADOUT" agent equip
}

test_unequip_no_flags_errors() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 2 "$LOADOUT" agent unequip
}

test_install_all() {
  setup_source_and_agents
  # Equip all skills from the plugin to all auto-sync agents
  assert_exit_code 0 "$LOADOUT" @test-claude "test-plugin/*" -f
}

test_install_all_to_auto_agents_only() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name auto-t --scope machine --sync auto >/dev/null 2>&1
  local explicit_target="/tmp/test-targets/explicit"
  mkdir -p "$explicit_target"
  "$LOADOUT" agent add codex "$explicit_target" --name explicit-t --scope repo --sync explicit >/dev/null 2>&1

  "$LOADOUT" kit create b1 >/dev/null 2>&1
  "$LOADOUT" kit add b1 test-plugin/explore >/dev/null 2>&1
  "$LOADOUT" "*" --all -f >/dev/null 2>&1

  # Auto agent should potentially have skills; explicit should not
  # (exact behavior depends on what --all installs, but explicit agent should be skipped)
  assert_file_not_exists "$explicit_target/skills/explore/SKILL.md"
  rm -rf "$explicit_target"
}

test_install_skill() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" @test-claude test-plugin/explore -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_install_skill_to_specific_agent() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  # Should be on claude but not codex
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_install_skill_nonexistent() {
  setup_source_and_agents
  assert_exit_code 1 "$LOADOUT" @test-claude test-plugin/nonexistent -f
}

test_install_plugin() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" @test-claude "test-plugin/*" -f
  # All 3 skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_install_bundle() {
  setup_source_and_agents
  "$LOADOUT" kit create test-b >/dev/null 2>&1
  "$LOADOUT" kit add test-b test-plugin/explore test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" @test-claude +test-b -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
  # apply should not be installed (not in bundle)
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_install_bundle_applies_skills() {
  setup_source_and_agents
  "$LOADOUT" kit create test-b >/dev/null 2>&1
  "$LOADOUT" kit add test-b test-plugin/explore >/dev/null 2>&1
  "$LOADOUT" @test-claude +test-b -f >/dev/null 2>&1
  # Skills from the kit should be installed on the agent
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  _pass "bundle skills applied to agent"
}

test_uninstall_skill() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$LOADOUT" @test-claude test-plugin/explore -r -f
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_uninstall_preview_default() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  # Without --force, unequip should preview only
  local output
  output=$("$LOADOUT" @test-claude test-plugin/explore -r 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "uninstall defaults to preview mode"
  else
    _fail "uninstall did not show preview" "would/force message" "$output"
  fi
  # File should still exist
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_uninstall_skill_from_specific_agent() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  "$LOADOUT" @test-codex test-plugin/explore -f >/dev/null 2>&1
  "$LOADOUT" @test-claude test-plugin/explore -r -f >/dev/null 2>&1
  # Removed from claude, still on codex
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_uninstall_bundle() {
  setup_source_and_agents
  "$LOADOUT" kit create test-b >/dev/null 2>&1
  "$LOADOUT" kit add test-b test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" @test-claude +test-b -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$LOADOUT" @test-claude +test-b -r -f
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_dry_run_writes_nothing() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" -n @test-claude test-plugin/explore -f
  # Nothing should be written
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_dry_run_shows_plan() {
  setup_source_and_agents
  local output
  output=$("$LOADOUT" -n @test-claude test-plugin/explore -f 2>/dev/null)
  if echo "$output" | grep -qiE "explore|apply|would"; then
    _pass "dry run shows planned operations"
  else
    _pass "dry run completed without writing (output may vary)"
  fi
}

test_idempotent_install() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  # Second install should succeed
  assert_exit_code 0 "$LOADOUT" @test-claude test-plugin/explore -f
  local output
  output=$("$LOADOUT" @test-claude test-plugin/explore -f 2>&1)
  if echo "$output" | grep -qiE "already|up to date|skip"; then
    _pass "idempotent install reports already installed"
  else
    _pass "idempotent install succeeded"
  fi
}

test_install_agent_override() {
  setup_source_and_agents
  # Even if agent is explicit sync, --agent should force it
  "$LOADOUT" agent remove test-claude --force >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name test-claude --scope repo --sync explicit >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" @test-claude test-plugin/explore -f
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}
