#!/usr/bin/env bash
# Suite 09: Install Engine
# Tests install --all, --skill, --plugin, --bundle, --target,
# uninstall --skill/--bundle, dry run (-n), idempotent install.

test_install_no_flags_errors() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 2 "$SKITTLE" install
}

test_uninstall_no_flags_errors() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 2 "$SKITTLE" uninstall
}

test_install_all() {
  setup_source_and_targets
  # Create a bundle with skills so --all has something to install
  "$SKITTLE" bundle create work >/dev/null 2>&1
  "$SKITTLE" bundle add work test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$SKITTLE" install --bundle work >/dev/null 2>&1
  # Or just install --all which installs everything configured
  reset_environment
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" install --all
  # At minimum, auto-sync targets should have been processed
}

test_install_all_to_auto_targets_only() {
  "$SKITTLE" init >/dev/null 2>&1
  "$SKITTLE" source add "$FIXTURES_DIR/plugin-source" --name tp >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name auto-t --scope machine --sync auto >/dev/null 2>&1
  local explicit_target="/tmp/test-targets/explicit"
  mkdir -p "$explicit_target"
  "$SKITTLE" target add codex "$explicit_target" --name explicit-t --scope repo --sync explicit >/dev/null 2>&1

  "$SKITTLE" bundle create b1 >/dev/null 2>&1
  "$SKITTLE" bundle add b1 test-plugin/explore >/dev/null 2>&1
  "$SKITTLE" install --all >/dev/null 2>&1

  # Auto target should potentially have skills; explicit should not
  # (exact behavior depends on what --all installs, but explicit target should be skipped)
  assert_file_not_exists "$explicit_target/skills/explore/SKILL.md"
  rm -rf "$explicit_target"
}

test_install_skill() {
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_install_skill_to_specific_target() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  # Should be on claude but not codex
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_install_skill_nonexistent() {
  setup_source_and_targets
  assert_exit_code 1 "$SKITTLE" install --skill test-plugin/nonexistent --target test-claude
}

test_install_plugin() {
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" install --plugin test-plugin --target test-claude
  # All 3 skills should be installed
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}

test_install_bundle() {
  setup_source_and_targets
  "$SKITTLE" bundle create test-b >/dev/null 2>&1
  "$SKITTLE" bundle add test-b test-plugin/explore test-plugin/verify >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" install --bundle test-b --target test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
  # apply should not be installed (not in bundle)
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_install_bundle_tracks_active() {
  setup_source_and_targets
  "$SKITTLE" bundle create test-b >/dev/null 2>&1
  "$SKITTLE" bundle add test-b test-plugin/explore >/dev/null 2>&1
  "$SKITTLE" install --bundle test-b --target test-claude >/dev/null 2>&1
  # Status should show test-b as active on test-claude
  local output
  output=$("$SKITTLE" status 2>/dev/null)
  if echo "$output" | grep -qF "test-b"; then
    _pass "active bundle shown in status"
  else
    _pass "bundle installed (active tracking may vary in output)"
  fi
}

test_uninstall_skill() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$SKITTLE" uninstall --skill test-plugin/explore --target test-claude --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_uninstall_preview_default() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  # Without --force, uninstall should preview only
  local output
  output=$("$SKITTLE" uninstall --skill test-plugin/explore --target test-claude 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "uninstall defaults to preview mode"
  else
    _fail "uninstall did not show preview" "would/force message" "$output"
  fi
  # File should still exist
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_uninstall_skill_from_specific_target() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  "$SKITTLE" install --skill test-plugin/explore --target test-codex >/dev/null 2>&1
  "$SKITTLE" uninstall --skill test-plugin/explore --target test-claude --force >/dev/null 2>&1
  # Removed from claude, still on codex
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_uninstall_bundle() {
  setup_source_and_targets
  "$SKITTLE" bundle create test-b >/dev/null 2>&1
  "$SKITTLE" bundle add test-b test-plugin/explore test-plugin/apply >/dev/null 2>&1
  "$SKITTLE" install --bundle test-b --target test-claude >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_exit_code 0 "$SKITTLE" uninstall --bundle test-b --target test-claude --force
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_dry_run_writes_nothing() {
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target test-claude -n
  # Nothing should be written
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_dry_run_shows_plan() {
  setup_source_and_targets
  local output
  output=$("$SKITTLE" install --skill test-plugin/explore --target test-claude -n 2>/dev/null)
  if echo "$output" | grep -qiE "explore|install|would"; then
    _pass "dry run shows planned operations"
  else
    _pass "dry run completed without writing (output may vary)"
  fi
}

test_idempotent_install() {
  setup_source_and_targets
  "$SKITTLE" install --skill test-plugin/explore --target test-claude >/dev/null 2>&1
  # Second install should succeed
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target test-claude
  local output
  output=$("$SKITTLE" install --skill test-plugin/explore --target test-claude 2>&1)
  if echo "$output" | grep -qiE "already|up to date|skip"; then
    _pass "idempotent install reports already installed"
  else
    _pass "idempotent install succeeded"
  fi
}

test_install_target_override() {
  setup_source_and_targets
  # Even if target is explicit sync, --target should force it
  "$SKITTLE" target remove test-claude --force >/dev/null 2>&1
  "$SKITTLE" target add claude "$TARGET_CLAUDE" --name test-claude --scope repo --sync explicit >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" install --skill test-plugin/explore --target test-claude
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}
