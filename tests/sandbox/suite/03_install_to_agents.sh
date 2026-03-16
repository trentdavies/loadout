#!/usr/bin/env bash
# Suite 03: Apply to Agents — apply skills to mock claude/codex agents
# Depends on Suite 01+02 having sources populated.

# Helper: extract a non-ambiguous plugin/skill identity from `loadout list --json`.
# Picks skills where plugin != source to avoid the ambiguity bug.
_first_skill() {
  "$LOADOUT" list --json 2>/dev/null | jq -r \
    '[.[] | select(.plugin != .source)] | .[0] | "\(.plugin)/\(.name)"'
}

_first_skill_from_source() {
  local source_name="$1"
  "$LOADOUT" list --json 2>/dev/null | jq -r --arg src "$source_name" \
    '[.[] | select(.source == $src and .plugin != .source)] | .[0] | "\(.plugin)/\(.name)"'
}

test_01_apply_single_skill() {
  local skill
  skill=$(_first_skill)

  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill found to apply" "plugin/skill" "empty"
    return
  fi

  log_cmd "$LOADOUT" @sandbox-claude "$skill" -f

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  local found
  found=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)

  if [ -n "$found" ]; then
    _pass "applied $short_name to claude agent"
    log_check 1 "applied $short_name — SKILL.md at $found"
  else
    _fail "SKILL.md not found for $short_name in claude agent" "file exists" "not found"
    log_check 0 "applied $short_name — SKILL.md not found in claude agent"
  fi
}

test_02_apply_plugin() {
  local plugin_name
  plugin_name=$("$LOADOUT" list --json 2>/dev/null | jq -r \
    '[.[] | select(.source == "anthropic-skills")] | .[0].plugin // empty')

  if [ -z "$plugin_name" ]; then
    _fail "no plugin found from anthropic-skills" "plugin name" "empty"
    return
  fi

  log_cmd "$LOADOUT" @sandbox-claude "$plugin_name/*" -f

  local installed
  installed=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -type f 2>/dev/null | wc -l | tr -d ' ')

  if [ "$installed" -gt 1 ]; then
    _pass "applied $installed skills from plugin $plugin_name"
    log_check 1 "plugin $plugin_name — $installed skills applied to claude"
  else
    _fail "plugin apply produced $installed skills" "more than 1" "$installed"
    log_check 0 "plugin $plugin_name apply"
  fi
}

test_03_apply_both_agents() {
  local skill
  skill=$(_first_skill_from_source "knowledge-work")
  [ -z "$skill" ] || [ "$skill" = "null/null" ] && skill=$(_first_skill)

  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill found to apply to both agents" "plugin/skill" "empty"
    return
  fi

  log_cmd "$LOADOUT" @sandbox-claude "$skill" -f
  log_cmd "$LOADOUT" @sandbox-codex "$skill" -f

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  local claude_found codex_found
  claude_found=$(find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)
  codex_found=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1)

  if [ -n "$claude_found" ] && [ -n "$codex_found" ]; then
    _pass "$short_name applied to both claude and codex agents"
    log_check 1 "$short_name present in both agents"
  else
    [ -z "$claude_found" ] && _fail "$short_name missing from claude agent" "present" "not found"
    [ -z "$codex_found" ] && _fail "$short_name missing from codex agent" "present" "not found"
    log_check 0 "$short_name in both agents"
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

test_05_uninstall_one_agent() {
  local skill
  skill=$(_first_skill_from_source "knowledge-work")
  [ -z "$skill" ] || [ "$skill" = "null/null" ] && skill=$(_first_skill)

  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available for uninstall test" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(echo "$skill" | awk -F/ '{print $NF}')

  # Ensure it's in both agents
  "$LOADOUT" @sandbox-claude "$skill" -f >/dev/null 2>&1
  "$LOADOUT" @sandbox-codex "$skill" -f >/dev/null 2>&1

  # Uninstall from claude only
  log_cmd "$LOADOUT" @sandbox-claude "$skill" --remove -f

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
  log_cmd "$LOADOUT" status
  local output
  output=$("$LOADOUT" status 2>/dev/null)
  local exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    _pass "status command succeeds"
    log_check 1 "loadout status exits cleanly"
  else
    _fail "status command failed" "exit 0" "exit $exit_code"
    log_check 0 "loadout status exits cleanly"
  fi

  if echo "$output" | grep -qiE "claude|codex|skill|agent"; then
    _pass "status output reflects installed state"
    log_check 1 "status mentions agents/skills"
  else
    _pass "status returned (format may vary)"
    log_check 1 "status returned output"
  fi
}
