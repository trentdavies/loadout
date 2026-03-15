#!/usr/bin/env bash
# Suite 05: Overwrite Protection — confirm apply refuses to overwrite changed skills
# Depends on Suite 03 having applied skills to sandbox-claude.

_first_skill() {
  # Pick a skill from knowledge-work (unambiguous — not duplicated by local-skills)
  "$LOADOUT" list --json 2>/dev/null | jq -r \
    '[.[] | select(.source == "knowledge-work")] | .[0] | "\(.plugin)/\(.name)"'
}

_skill_short_name() {
  echo "$1" | awk -F/ '{print $NF}'
}

_find_installed_skill_md() {
  local short_name="$1"
  find "$SANDBOX_TARGET_CLAUDE" -name "SKILL.md" -path "*$short_name*" 2>/dev/null | head -1
}

test_01_apply_blocks_on_changed_skill() {
  local skill
  skill=$(_first_skill)
  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available for overwrite test" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(_skill_short_name "$skill")

  # Ensure skill is installed first
  "$LOADOUT" apply --force --skill "$skill" --agent sandbox-claude >/dev/null 2>&1

  local skill_file
  skill_file=$(_find_installed_skill_md "$short_name")
  if [ -z "$skill_file" ]; then
    _fail "skill not installed — cannot test overwrite protection" "SKILL.md exists" "not found"
    return
  fi

  # Tamper with the installed file to create a CHANGED state
  echo "# locally modified content" >> "$skill_file"

  # Apply WITHOUT --force — should be blocked
  local output exit_code
  output=$("$LOADOUT" apply --skill "$skill" --agent sandbox-claude 2>&1)
  exit_code=$?

  if [ "$exit_code" -ne 0 ]; then
    _pass "apply refused to overwrite changed skill (exit $exit_code)"
    log_check 1 "apply without --force blocked on CHANGED skill $short_name"
  else
    _fail "apply should have refused to overwrite" "non-zero exit" "exit $exit_code"
    log_check 0 "apply without --force should block on CHANGED skill"
  fi

  # Verify the local modification is still intact (no overwrite occurred)
  if grep -qF "locally modified content" "$skill_file"; then
    _pass "agent file was not overwritten"
    log_check 1 "local modification preserved after blocked apply"
  else
    _fail "agent file was overwritten despite no --force" "modification preserved" "modification lost"
    log_check 0 "local modification preserved"
  fi
}

test_02_unchanged_skill_applies_without_force() {
  local skill
  skill=$(_first_skill)
  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(_skill_short_name "$skill")

  # Force-apply to ensure agent matches source exactly
  "$LOADOUT" apply --force --skill "$skill" --agent sandbox-claude >/dev/null 2>&1

  # Apply again WITHOUT --force — should succeed (UNCHANGED)
  local output exit_code
  output=$("$LOADOUT" apply --skill "$skill" --agent sandbox-claude 2>&1)
  exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    _pass "unchanged skill accepted without --force"
    log_check 1 "apply without --force succeeds for UNCHANGED skill $short_name"
  else
    _fail "unchanged skill rejected" "exit 0" "exit $exit_code"
    log_check 0 "unchanged skill should pass without --force"
  fi
}

test_03_new_skill_applies_without_force() {
  local skill
  skill=$(_first_skill)
  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(_skill_short_name "$skill")

  # Remove the installed skill directory so it becomes NEW
  local skill_dir
  skill_dir=$(find "$SANDBOX_TARGET_CLAUDE" -type d -name "$short_name" 2>/dev/null | head -1)
  if [ -n "$skill_dir" ]; then
    rm -rf "$skill_dir"
  fi

  # Apply WITHOUT --force — should succeed (NEW skill)
  local output exit_code
  output=$("$LOADOUT" apply --skill "$skill" --agent sandbox-claude 2>&1)
  exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    _pass "new skill applied without --force"
    log_check 1 "apply without --force succeeds for NEW skill $short_name"
  else
    _fail "new skill rejected without --force" "exit 0" "exit $exit_code"
    log_check 0 "new skill should apply without --force"
  fi

  # Verify it was actually installed
  local skill_file
  skill_file=$(_find_installed_skill_md "$short_name")
  if [ -n "$skill_file" ]; then
    _pass "new skill SKILL.md written to agent"
    log_check 1 "SKILL.md present after applying NEW skill"
  else
    _fail "SKILL.md not found after apply" "file exists" "not found"
    log_check 0 "SKILL.md present after apply"
  fi
}

test_04_force_flag_overwrites_changed() {
  local skill
  skill=$(_first_skill)
  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(_skill_short_name "$skill")

  # Ensure skill is installed cleanly
  "$LOADOUT" apply --force --skill "$skill" --agent sandbox-claude >/dev/null 2>&1

  local skill_file
  skill_file=$(_find_installed_skill_md "$short_name")
  if [ -z "$skill_file" ]; then
    _fail "skill not installed for force overwrite test" "SKILL.md exists" "not found"
    return
  fi

  # Tamper with the installed file
  echo "# local edit that should be overwritten" >> "$skill_file"

  # Apply WITH --force — should overwrite
  log_cmd "$LOADOUT" apply --force --skill "$skill" --agent sandbox-claude

  # Verify the local modification is gone (overwritten by source)
  if grep -qF "local edit that should be overwritten" "$skill_file"; then
    _fail "force apply did not overwrite the changed file" "modification removed" "modification still present"
    log_check 0 "--force should overwrite changed content"
  else
    _pass "force apply overwrote changed skill"
    log_check 1 "--force restored source content, local edit removed"
  fi
}

test_05_error_suggests_force_or_interactive() {
  local skill
  skill=$(_first_skill)
  if [ -z "$skill" ] || [ "$skill" = "null/null" ]; then
    _fail "no skill available" "plugin/skill" "empty"
    return
  fi

  local short_name
  short_name=$(_skill_short_name "$skill")

  # Ensure skill is installed cleanly, then tamper
  "$LOADOUT" apply --force --skill "$skill" --agent sandbox-claude >/dev/null 2>&1

  local skill_file
  skill_file=$(_find_installed_skill_md "$short_name")
  if [ -z "$skill_file" ]; then
    _fail "skill not installed for error message test" "SKILL.md exists" "not found"
    return
  fi

  echo "# tampered for error message test" >> "$skill_file"

  local output
  output=$("$LOADOUT" apply --skill "$skill" --agent sandbox-claude 2>&1)

  local has_force=0 has_interactive=0
  echo "$output" | grep -qF -- "--force" && has_force=1
  echo "$output" | grep -qE -- "-i|--interactive" && has_interactive=1

  if [ "$has_force" -eq 1 ] && [ "$has_interactive" -eq 1 ]; then
    _pass "error message suggests --force and -i"
    log_check 1 "conflict error includes --force and -i suggestions"
  elif [ "$has_force" -eq 1 ] || [ "$has_interactive" -eq 1 ]; then
    _pass "error message suggests at least one resolution flag"
    log_check 1 "conflict error includes at least one flag suggestion"
  else
    _fail "error message missing resolution suggestions" "--force and -i mentioned" "not found in: $output"
    log_check 0 "conflict error should suggest --force or -i"
  fi
}
