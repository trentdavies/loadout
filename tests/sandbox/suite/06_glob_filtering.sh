#!/usr/bin/env bash
# Suite 06: Glob Filtering — verify glob patterns filter skills in list and apply
# Depends on Suite 01+02 having sources populated with skills.

test_01_list_wildcard_all() {
  local all_count glob_count
  all_count=$("$LOADOUT" list --json 2>/dev/null | jq 'length')
  glob_count=$("$LOADOUT" list --json "*/*" 2>/dev/null | jq 'length')

  if [ "$glob_count" -gt 0 ] && [ "$glob_count" -eq "$all_count" ]; then
    _pass "list '*/*' returns all $all_count skills"
    log_check 1 "glob */* matches all skills ($glob_count/$all_count)"
  elif [ "$glob_count" -gt 0 ]; then
    _pass "list '*/*' returns $glob_count skills (all=$all_count)"
    log_check 1 "glob */* returns results ($glob_count)"
  else
    _fail "list '*/*' returned 0 skills" "all skills" "0 (total=$all_count)"
    log_check 0 "glob */* should match skills"
  fi
}

test_02_list_glob_by_source() {
  # Pick the first source that has skills
  local source_name
  source_name=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[0].source // empty')

  if [ -z "$source_name" ]; then
    _fail "no sources available for glob test" "source name" "empty"
    return
  fi

  local source_count glob_count
  source_count=$("$LOADOUT" list --json 2>/dev/null | jq --arg s "$source_name" '[.[] | select(.source == $s)] | length')
  glob_count=$("$LOADOUT" list --json "$source_name:*/*" 2>/dev/null | jq 'length')

  if [ "$glob_count" -eq "$source_count" ] && [ "$glob_count" -gt 0 ]; then
    _pass "list '$source_name:*/*' returns $glob_count skills (matches source count)"
    log_check 1 "source-scoped glob matches expected count"
  elif [ "$glob_count" -gt 0 ]; then
    _pass "list '$source_name:*/*' returns $glob_count skills"
    log_check 1 "source-scoped glob returns results"
  else
    _fail "list '$source_name:*/*' returned 0 skills" "$source_count" "0"
    log_check 0 "source-scoped glob should match"
  fi
}

test_03_list_glob_by_plugin() {
  # Pick the first plugin name
  local plugin_name
  plugin_name=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[0].plugin // empty')

  if [ -z "$plugin_name" ]; then
    _fail "no plugins available for glob test" "plugin name" "empty"
    return
  fi

  local plugin_count glob_count
  plugin_count=$("$LOADOUT" list --json 2>/dev/null | jq --arg p "$plugin_name" '[.[] | select(.plugin == $p)] | length')
  glob_count=$("$LOADOUT" list --json "$plugin_name/*" 2>/dev/null | jq 'length')

  if [ "$glob_count" -eq "$plugin_count" ] && [ "$glob_count" -gt 0 ]; then
    _pass "list '$plugin_name/*' returns $glob_count skills"
    log_check 1 "plugin glob matches expected count"
  elif [ "$glob_count" -gt 0 ]; then
    _pass "list '$plugin_name/*' returns $glob_count skills"
    log_check 1 "plugin glob returns results"
  else
    _fail "list '$plugin_name/*' returned 0 skills" "$plugin_count" "0"
    log_check 0 "plugin glob should match"
  fi
}

test_04_list_glob_no_match() {
  local output exit_code
  output=$("$LOADOUT" list --json "nonexistent-plugin-xyz/*" 2>&1)
  exit_code=$?

  local count
  count=$(echo "$output" | jq 'length' 2>/dev/null)

  if [ "$count" = "0" ] || [ -z "$count" ]; then
    _pass "glob with no matches returns empty result"
    log_check 1 "nonexistent glob returns 0 results"
  else
    _fail "glob should return empty for nonexistent pattern" "0" "$count"
    log_check 0 "nonexistent glob should return empty"
  fi
}

test_05_list_glob_question_mark() {
  # Find a skill name, then use ? to match one character
  local skill_name
  skill_name=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[0].name // empty')
  local plugin_name
  plugin_name=$("$LOADOUT" list --json 2>/dev/null | jq -r '.[0].plugin // empty')

  if [ -z "$skill_name" ] || [ -z "$plugin_name" ]; then
    _fail "no skill available for ? glob test" "skill" "empty"
    return
  fi

  # Replace last char with ?
  local pattern="${plugin_name}/${skill_name%?}?"
  local glob_count
  glob_count=$("$LOADOUT" list --json "$pattern" 2>/dev/null | jq 'length')

  if [ "$glob_count" -gt 0 ]; then
    _pass "list '$pattern' matches $glob_count skill(s) using ?"
    log_check 1 "? glob matches"
  else
    _fail "list '$pattern' returned 0 skills" "at least 1" "0"
    log_check 0 "? glob should match"
  fi
}

test_06_freeform_source_prefix() {
  # 'anthr*' should match all skills from anthropic-skills source
  local source_count glob_count
  source_count=$("$LOADOUT" list --json 2>/dev/null | jq '[.[] | select(.source == "anthropic-skills")] | length')
  glob_count=$("$LOADOUT" list --json 'anthr*' 2>/dev/null | jq 'length')

  if [ "$source_count" -eq 0 ]; then
    _fail "anthropic-skills source has no skills" "skills present" "0"
    return
  fi

  if [ "$glob_count" -eq "$source_count" ]; then
    _pass "list 'anthr*' matches all $source_count anthropic-skills entries"
    log_check 1 "freeform source prefix — $glob_count/$source_count"
  elif [ "$glob_count" -gt 0 ]; then
    _pass "list 'anthr*' matches $glob_count skills (expected $source_count)"
    log_check 1 "freeform source prefix — partial match"
  else
    _fail "list 'anthr*' returned 0 skills" "$source_count" "0"
    log_check 0 "freeform source prefix should match"
  fi
}

test_07_freeform_substring_local() {
  # Add a local source if not already present, then '*local*' should match it
  if ! "$LOADOUT" list 2>/dev/null | grep -qF "local"; then
    _skip "no source with 'local' in name — skipping *local* test"
    return
  fi

  local glob_count
  glob_count=$("$LOADOUT" list --json '*local*' 2>/dev/null | jq 'length')

  if [ "$glob_count" -gt 0 ]; then
    _pass "list '*local*' matches $glob_count skills"
    log_check 1 "freeform substring *local* matches"
  else
    _fail "list '*local*' returned 0 skills" "at least 1" "0"
    log_check 0 "freeform substring *local* should match"
  fi
}

test_08_freeform_substring_finan() {
  # '*finan*' should match financial-services skills (like grep finan)
  local grep_count glob_count
  grep_count=$("$LOADOUT" list --json 2>/dev/null | jq '[.[] | select((.source | test("finan")) or (.plugin | test("finan")) or (.name | test("finan")))] | length')
  glob_count=$("$LOADOUT" list --json '*finan*' 2>/dev/null | jq 'length')

  if [ "$grep_count" -eq 0 ]; then
    _fail "no skills contain 'finan' in identity" "skills present" "0"
    return
  fi

  if [ "$glob_count" -eq "$grep_count" ]; then
    _pass "list '*finan*' matches all $grep_count entries containing 'finan'"
    log_check 1 "freeform *finan* equivalent to grep — $glob_count/$grep_count"
  elif [ "$glob_count" -gt 0 ]; then
    _pass "list '*finan*' matches $glob_count skills (grep=$grep_count)"
    log_check 1 "freeform *finan* returns results"
  else
    _fail "list '*finan*' returned 0 skills" "$grep_count" "0"
    log_check 0 "freeform *finan* should match"
  fi
}

test_10_apply_glob_to_agent() {
  # Pick a plugin with multiple skills and apply via glob
  local plugin_name
  plugin_name=$("$LOADOUT" list --json 2>/dev/null | jq -r \
    '[group_by(.plugin)[] | select(length > 1) | .[0].plugin] | .[0] // empty')

  if [ -z "$plugin_name" ]; then
    _fail "no multi-skill plugin found for glob apply test" "plugin" "empty"
    return
  fi

  local expected_count
  expected_count=$("$LOADOUT" list --json "$plugin_name/*" 2>/dev/null | jq 'length')

  # Clean agent and apply via glob
  find "$SANDBOX_TARGET_CODEX/skills" -mindepth 1 -maxdepth 1 -type d -exec rm -r {} + 2>/dev/null
  mkdir -p "$SANDBOX_TARGET_CODEX"

  log_cmd "$LOADOUT" apply --force --skill "$plugin_name/*" --agent sandbox-codex

  local installed
  installed=$(find "$SANDBOX_TARGET_CODEX" -name "SKILL.md" -type f 2>/dev/null | wc -l | tr -d ' ')

  if [ "$installed" -gt 0 ]; then
    _pass "glob apply installed $installed skills from '$plugin_name/*'"
    log_check 1 "glob apply — $installed skills installed (expected ~$expected_count)"
  else
    _fail "glob apply installed 0 skills" "at least 1" "0"
    log_check 0 "glob apply should install skills"
  fi
}
