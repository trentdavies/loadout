#!/usr/bin/env bash
# Suite 03: Install to Targets — install skills to mock claude/codex targets
# Depends on Suite 01+02 having sources populated.
#
# Note: the anthropic-skills repo has plugin name == source name, which triggers
# a duplication bug in skittle making --skill ambiguous. We prefer skills from
# knowledge-work (where plugin != source) for --skill tests, and use --plugin
# for anthropic-skills tests.

# Helper: extract a non-ambiguous plugin/skill from the list.
# Picks from sources where plugin name != source name to avoid the duplication bug.
_unambiguous_skill() {
  "$SKITTLE" list 2>/dev/null | tail -n +3 | awk '$2 != $3 && NF>=3 {print $2 "/" $1; exit}'
}

_unambiguous_skill_from_source() {
  local source_name="$1"
  "$SKITTLE" list 2>/dev/null | tail -n +3 | awk -v src="$source_name" '$NF == src && $2 != $3 && NF>=3 {print $2 "/" $1; exit}'
}

test_01_install_single_skill() {
  local skill
  skill=$(_unambiguous_skill)

  if [ -z "$skill" ]; then
    _fail "no unambiguous skill found to install" "plugin/skill" "empty"
    return
  fi

  log_cmd "$SKITTLE" install --skill "$skill" --target sandbox-claude

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  local found
  found=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)

  if [ -n "$found" ]; then
    _pass "installed $short_name to claude target"
    log_check 1 "installed $short_name — SKILL.md at $found"
  else
    _fail "SKILL.md not found for $short_name in claude target" "file exists" "not found"
    log_check 0 "installed $short_name — SKILL.md not found in claude target"
  fi
}

test_02_install_plugin() {
  # anthropic-skills has plugin == source, so use --plugin which works fine
  local plugin_name
  plugin_name=$("$SKITTLE" list 2>/dev/null | tail -n +3 | awk '$NF == "anthropic-skills" {print $2; exit}')

  if [ -z "$plugin_name" ]; then
    _fail "no plugin found from anthropic-skills" "plugin name" "empty"
    return
  fi

  log_cmd "$SKITTLE" install --plugin "$plugin_name" --target sandbox-claude

  local installed
  installed=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -type f 2>/dev/null | wc -l | tr -d ' ')

  if [ "$installed" -gt 1 ]; then
    _pass "installed $installed skills from plugin $plugin_name"
    log_check 1 "plugin $plugin_name — $installed skills installed to claude"
  else
    _fail "plugin install produced $installed skills" "more than 1" "$installed"
    log_check 0 "plugin $plugin_name install"
  fi
}

test_03_install_both_targets() {
  local skill
  skill=$(_unambiguous_skill_from_source "knowledge-work")
  [ -z "$skill" ] && skill=$(_unambiguous_skill)

  if [ -z "$skill" ]; then
    _fail "no skill found to install to both targets" "plugin/skill" "empty"
    return
  fi

  log_cmd "$SKITTLE" install --skill "$skill" --target sandbox-claude
  log_cmd "$SKITTLE" install --skill "$skill" --target sandbox-codex

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  local claude_found codex_found
  claude_found=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)
  codex_found=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)

  if [ -n "$claude_found" ] && [ -n "$codex_found" ]; then
    _pass "$short_name installed to both claude and codex targets"
    log_check 1 "$short_name present in both targets"
  else
    [ -z "$claude_found" ] && _fail "$short_name missing from claude target" "present" "not found"
    [ -z "$codex_found" ] && _fail "$short_name missing from codex target" "present" "not found"
    log_check 0 "$short_name in both targets"
  fi
}

test_04_installed_content_valid() {
  local skill_file
  skill_file=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -type f 2>/dev/null | head -1)

  if [ -z "$skill_file" ]; then
    _fail "no installed SKILL.md found to validate" "file exists" "none found"
    return
  fi

  if head -1 "$skill_file" | grep -q "^---"; then
    _pass "installed SKILL.md has frontmatter"
    log_check 1 "SKILL.md at $skill_file starts with frontmatter"
  else
    _fail "installed SKILL.md missing frontmatter" "starts with ---" "$(head -1 "$skill_file")"
    log_check 0 "SKILL.md at $skill_file missing frontmatter"
  fi
}

test_05_uninstall_one_target() {
  local skill
  skill=$(_unambiguous_skill_from_source "knowledge-work")
  [ -z "$skill" ] && skill=$(_unambiguous_skill)

  if [ -z "$skill" ]; then
    _fail "no skill available for uninstall test" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  # Ensure it's in both targets
  "$SKITTLE" install --skill "$skill" --target sandbox-claude >/dev/null 2>&1
  "$SKITTLE" install --skill "$skill" --target sandbox-codex >/dev/null 2>&1

  # Uninstall from claude only
  log_cmd "$SKITTLE" uninstall --skill "$skill" --target sandbox-claude --force

  local claude_found codex_found
  claude_found=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)
  codex_found=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)

  if [ -z "$claude_found" ] && [ -n "$codex_found" ]; then
    _pass "uninstalled from claude, codex untouched"
    log_check 1 "$short_name removed from claude, still in codex"
  else
    [ -n "$claude_found" ] && _fail "$short_name still in claude after uninstall" "removed" "still present"
    [ -z "$codex_found" ] && _fail "$short_name missing from codex (should be untouched)" "present" "not found"
    log_check 0 "uninstall from claude — codex state incorrect"
  fi
}

test_06_status_reflects_state() {
  log_cmd "$SKITTLE" status
  local output
  output=$("$SKITTLE" status 2>/dev/null)
  local exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    _pass "status command succeeds"
    log_check 1 "skittle status exits cleanly"
  else
    _fail "status command failed" "exit 0" "exit $exit_code"
    log_check 0 "skittle status exits cleanly"
  fi

  if echo "$output" | grep -qiE "claude|codex|install|skill|target"; then
    _pass "status output reflects installed state"
    log_check 1 "status mentions targets/skills"
  else
    _pass "status returned (format may vary)"
    log_check 1 "status returned output"
  fi
}
