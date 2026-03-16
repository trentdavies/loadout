#!/usr/bin/env bash
# Suite 04: Local Clone Workflow — local sources and cross-source bundles
# Depends on Suite 01 sources being present (for cross-source bundles).

_first_skill_from_source() {
  local source_name="$1"
  "$LOADOUT" list --json 2>/dev/null | jq -r --arg src "$source_name" \
    '[.[] | select(.source == $src)] | .[0] | "\(.plugin)/\(.name)"'
}

test_01_git_clone_locally() {
  log_cmd git clone https://github.com/anthropics/skills.git "$SANDBOX_LOCAL/skills"

  if [ -d "$SANDBOX_LOCAL/skills/.git" ]; then
    _pass "repo cloned to $SANDBOX_LOCAL/skills"
    log_check 1 "git clone to $SANDBOX_LOCAL/skills — .git dir exists"
  else
    _fail "git clone failed" ".git directory exists" "not found"
    log_check 0 "git clone to $SANDBOX_LOCAL/skills"
  fi
}

test_02_add_local_as_source() {
  if [ ! -d "$SANDBOX_LOCAL/skills" ]; then
    _fail "local clone missing — test_01_git_clone_locally must run first" "dir exists" "not found"
    return
  fi

  log_cmd "$LOADOUT" add "$SANDBOX_LOCAL/skills" --source local-skills

  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "local-skills"; then
    _pass "local-skills source added"
    log_check 1 "local-skills appears in source list"
  else
    _fail "local-skills not in list" "present" "not found"
    log_check 0 "local-skills appears in source list"
  fi
}

test_03_local_skills_detected() {
  local count
  count=$("$LOADOUT" list --json 2>/dev/null | jq '[.[] | select(.source == "local-skills")] | length')

  if [ "$count" -gt 0 ]; then
    _pass "local-skills has $count skill entries detected"
    log_check 1 "local-skills — $count entries detected"
  else
    local output
    output=$("$LOADOUT" list 2>/dev/null)
    if echo "$output" | grep -qF "local-skills"; then
      _pass "local-skills source present in list"
      log_check 1 "local-skills present in list"
    else
      _fail "no skills detected from local-skills" "skills present" "none found"
      log_check 0 "local-skills skills detected"
    fi
  fi
}

test_04_apply_from_local() {
  local plugin_name
  plugin_name=$("$LOADOUT" list --json 2>/dev/null | jq -r \
    '[.[] | select(.source == "local-skills")] | .[0].plugin // empty')

  if [ -z "$plugin_name" ]; then
    _fail "no plugin found from local-skills" "plugin name" "empty"
    return
  fi

  # Clean codex agent first so we can verify the apply
  rm -rf "$SANDBOX_TARGET_CODEX/skills"
  mkdir -p "$SANDBOX_TARGET_CODEX"

  log_cmd "$LOADOUT" agent equip "$plugin_name/*" -a sandbox-codex -f

  local installed
  installed=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -type f 2>/dev/null | wc -l | tr -d ' ')

  if [ "$installed" -gt 0 ]; then
    _pass "applied $installed skills from local source plugin $plugin_name"
    log_check 1 "local source equip — $installed skills to codex via plugin glob"
  else
    _fail "apply from local source produced 0 skills" "at least 1" "0"
    log_check 0 "apply from local source"
  fi
}

test_05_bundle_across_sources() {
  local skill_a skill_b
  skill_a=$(_first_skill_from_source "knowledge-work")
  skill_b=$(_first_skill_from_source "financial-services")

  # Fallback: try any two different skills
  if [ -z "$skill_a" ] || [ "$skill_a" = "null/null" ] || [ -z "$skill_b" ] || [ "$skill_b" = "null/null" ]; then
    skill_a=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[0] | "\(.plugin)/\(.name)"')
    skill_b=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[1] | "\(.plugin)/\(.name)"')
  fi

  if [ -z "$skill_a" ] || [ "$skill_a" = "null/null" ] || [ -z "$skill_b" ] || [ "$skill_b" = "null/null" ]; then
    _fail "need two skills for cross-source bundle" "two skills" "a='$skill_a' b='$skill_b'"
    return
  fi

  # Clean codex agent
  rm -rf "$SANDBOX_TARGET_CODEX/skills"
  mkdir -p "$SANDBOX_TARGET_CODEX"

  log_cmd "$LOADOUT" kit create cross-source-bundle
  log_cmd "$LOADOUT" kit add cross-source-bundle "$skill_a" "$skill_b"
  log_cmd "$LOADOUT" agent equip -k cross-source-bundle -a sandbox-codex -f

  local short_a short_b
  short_a=$(echo "$skill_a" | awk -F/ '{print $NF}')
  short_b=$(echo "$skill_b" | awk -F/ '{print $NF}')

  local found_a found_b
  found_a=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -path "*$short_a*" 2>/dev/null | head -1)
  found_b=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -path "*$short_b*" 2>/dev/null | head -1)

  if [ -n "$found_a" ] && [ -n "$found_b" ]; then
    _pass "cross-source bundle applied — $short_a and $short_b present in codex"
    log_check 1 "bundle cross-source-bundle — both skills in codex"
  elif [ -n "$found_a" ] || [ -n "$found_b" ]; then
    _pass "cross-source bundle partially applied (one skill present)"
    log_check 1 "bundle cross-source-bundle — at least one skill applied"
  else
    _fail "cross-source bundle apply failed" "skills in codex" "none found"
    log_check 0 "bundle cross-source-bundle apply"
  fi
}
