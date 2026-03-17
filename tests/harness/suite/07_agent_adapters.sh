#!/usr/bin/env bash
# Suite 07: Agent Adapters
# Tests claude adapter, codex adapter, custom TOML adapter, unknown format error.

test_claude_adapter_installs_skill_md() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
}

test_claude_adapter_copies_scripts_dir() {
  setup_source_and_agents
  # apply skill has a scripts/ directory
  "$LOADOUT" @test-claude test-plugin/apply -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_dir_exists "$TARGET_CLAUDE/skills/apply/scripts"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/scripts/run.sh"
}

test_claude_adapter_skill_content_matches() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  # Installed SKILL.md should contain the original frontmatter
  assert_file_contains "$TARGET_CLAUDE/skills/explore/SKILL.md" "name: explore"
  assert_file_contains "$TARGET_CLAUDE/skills/explore/SKILL.md" "description:"
}

test_codex_adapter_installs_skill_md() {
  setup_source_and_agents
  "$LOADOUT" @test-codex test-plugin/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CODEX/skills/explore/SKILL.md"
}

test_codex_adapter_copies_scripts_dir() {
  setup_source_and_agents
  "$LOADOUT" @test-codex test-plugin/apply -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CODEX/skills/apply/SKILL.md"
  assert_dir_exists "$TARGET_CODEX/skills/apply/scripts"
}

test_claude_adapter_uninstalls_cleanly() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  "$LOADOUT" @test-claude test-plugin/explore -r -f >/dev/null 2>&1
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  # The skill directory itself should be removed
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore"
}

test_custom_toml_adapter() {
  "$LOADOUT" init >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1

  # Define a custom adapter in the config
  local config_file="$XDG_DATA_HOME/equip/equip.toml"
  cat >> "$config_file" <<'TOML'

[adapter.custom-agent]
skill_dir = "prompts/{name}"
skill_file = "SKILL.md"
format = "agentskills"
copy_dirs = []
TOML

  # Add an agent using the custom adapter
  local custom_target="/tmp/test-targets/custom"
  mkdir -p "$custom_target"
  "$LOADOUT" agent add custom-agent "$custom_target" --name test-custom >/dev/null 2>&1
  "$LOADOUT" @test-custom test-plugin/explore -f >/dev/null 2>&1

  # Should use the custom path template
  assert_file_exists "$custom_target/prompts/explore/SKILL.md"

  # copy_dirs is empty, so scripts should NOT be copied
  "$LOADOUT" @test-custom test-plugin/apply -f >/dev/null 2>&1
  assert_file_exists "$custom_target/prompts/apply/SKILL.md"
  assert_file_not_exists "$custom_target/prompts/apply/scripts"

  rm -rf "$custom_target"
}

test_custom_adapter_unknown_format_error() {
  "$LOADOUT" init >/dev/null 2>&1

  # Define adapter with unsupported format
  local config_file="$XDG_DATA_HOME/equip/equip.toml"
  cat >> "$config_file" <<'TOML'

[adapter.bad-format]
skill_dir = "skills/{name}"
skill_file = "SKILL.md"
format = "mdc"
copy_dirs = []
TOML

  local bad_target="/tmp/test-targets/bad-fmt"
  mkdir -p "$bad_target"

  # Adding the agent might succeed, but installing should fail on unknown format
  "$LOADOUT" agent add bad-format "$bad_target" --name test-bad-fmt >/dev/null 2>&1
  "$LOADOUT" add "$FIXTURES_DIR/plugin-source" --source tp >/dev/null 2>&1

  local output
  output=$("$LOADOUT" @test-bad-fmt test-plugin/explore -f 2>&1)
  local exit_code=$?

  if [ "$exit_code" -ne 0 ] || echo "$output" | grep -qiE "unknown.*format|unsupported|mdc"; then
    _pass "unknown format rejected"
  else
    _fail "unknown format was accepted" "error about format 'mdc'" "$output"
  fi

  rm -rf "$bad_target"
}

test_multiple_skills_installed_to_same_agent() {
  setup_source_and_agents
  "$LOADOUT" @test-claude test-plugin/explore -f >/dev/null 2>&1
  "$LOADOUT" @test-claude test-plugin/apply -f >/dev/null 2>&1
  "$LOADOUT" @test-claude test-plugin/verify -f >/dev/null 2>&1
  assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
}
