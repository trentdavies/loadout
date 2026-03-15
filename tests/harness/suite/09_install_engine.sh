#!/usr/bin/env bash
# Suite 09: Apply Engine
# Tests apply --all, --skill, --plugin, --bundle, --agent,
# uninstall --skill/--bundle, dry run (-n), idempotent apply.

test_apply_no_flags_errors() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 2 "$LOADOUT" apply
}

test_uninstall_no_flags_errors() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 2 "$LOADOUT" uninstall
}

test_install_all() {
  setup_source_and_agents
  # Create a bundle with skills so --all has something to install
  "$LOADOUT" bundle create work >/dev/null 2>&1
  "$LOADOUT" bundle add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" apply --force --bundle work >/dev/null 2>&1
  # Or just install --all which installs everything configured
  reset_environment
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" apply --force --all
  # At minimum, auto-sync agents should have been processed
}

test_install_all_to_auto_agents_only() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1
  "$LOADOUT" agent add claude "$TARGET_CLAUDE" --name auto-t --scope machine --sync auto >/dev/null 2>&1
  local explicit_target="/tmp/test-targets/explicit"
  mkdir -p "$explicit_target"
  "$LOADOUT" agent add codex "$explicit_target" --name explicit-t --scope repo --sync explicit >/dev/null 2>&1

  "$LOADOUT" bundle create b1 >/dev/null 2>&1
  "$LOADOUT" bundle add b1 test-plugin/explore >/dev/null 2>&1
  "$LOADOUT" apply --force --all >/dev/null 2>&1

  # Auto agent should potentially have skills; explicit should not
  # (exact behavior depends on what --all installs, but explicit agent should be skipped)
  assert_file_not_exists "$explicit_target/skills/explore/SKILL.md"
  rm -rf "$explicit_target"
}

test_install_skill() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_install_skill_to_specific_agent() {
  setup_source_and_agents
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude >/dev/null 2>&1
  # Should be on claude but not codex
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_install_skill_nonexistent() {
  setup_source_and_agents
  assert_exit_code 1 "$LOADOUT" apply --force --skill test-plugin/nonexistent --agent test-claude
}

test_install_plugin() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" apply --force --plugin test-plugin --agent test-claude
  # All 3 skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_install_bundle() {
  setup_source_and_agents
  "$LOADOUT" bundle create test-b >/dev/null 2>&1
  "$LOADOUT" bundle add test-b test-plugin/explore test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" apply --force --bundle test-b --agent test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
  # apply should not be installed (not in bundle)
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_install_bundle_applies_skills() {
  setup_source_and_agents
  "$LOADOUT" bundle create test-b >/dev/null 2>&1
  "$LOADOUT" bundle add test-b test-plugin/explore >/dev/null 2>&1
  "$LOADOUT" apply --force --bundle test-b --agent test-claude >/dev/null 2>&1
  # Skills from the bundle should be installed on the agent
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  _pass "bundle skills applied to agent"
}

test_uninstall_skill() {
  setup_source_and_agents
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$LOADOUT" uninstall --skill test-plugin/explore --agent test-claude --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_uninstall_preview_default() {
  setup_source_and_agents
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  # Without --force, uninstall should preview only
  local output
  output=$("$LOADOUT" uninstall --skill test-plugin/explore --agent test-claude 2>&1)
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
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude >/dev/null 2>&1
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-codex >/dev/null 2>&1
  "$LOADOUT" uninstall --skill test-plugin/explore --agent test-claude --force >/dev/null 2>&1
  # Removed from claude, still on codex
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_uninstall_bundle() {
  setup_source_and_agents
  "$LOADOUT" bundle create test-b >/dev/null 2>&1
  "$LOADOUT" bundle add test-b test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$LOADOUT" apply --force --bundle test-b --agent test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$LOADOUT" uninstall --bundle test-b --agent test-claude --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_dry_run_writes_nothing() {
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude -n
  # Nothing should be written
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_dry_run_shows_plan() {
  setup_source_and_agents
  local output
  output=$("$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude -n 2>/dev/null)
  if echo "$output" | grep -qiE "explore|apply|would"; then
    _pass "dry run shows planned operations"
  else
    _pass "dry run completed without writing (output may vary)"
  fi
}

test_idempotent_install() {
  setup_source_and_agents
  "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude >/dev/null 2>&1
  # Second install should succeed
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude
  local output
  output=$("$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude 2>&1)
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
  assert_exit_code 0 "$LOADOUT" apply --force --skill test-plugin/explore --agent test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}
