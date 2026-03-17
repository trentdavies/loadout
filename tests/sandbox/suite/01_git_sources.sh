#!/usr/bin/env bash
# Suite 01: Git Sources — clone real Anthropic repos via network

test_01_add_anthropic_skills() {
  log_cmd "$LOADOUT" add https://github.com/anthropics/skills.git --source anthropic-skills
  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "anthropic-skills"; then
    _pass "anthropic-skills source added"
    log_check 1 "anthropic-skills appears in source list"
  else
    _fail "anthropic-skills not in list" "present" "not found"
    log_check 0 "anthropic-skills appears in source list"
  fi
}

test_02_add_knowledge_work() {
  log_cmd "$LOADOUT" add https://github.com/anthropics/knowledge-work-plugins.git --source knowledge-work
  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "knowledge-work"; then
    _pass "knowledge-work source added"
    log_check 1 "knowledge-work appears in source list"
  else
    _fail "knowledge-work not in plain text list output" "present" "not found (grep -qF on list output)"
    log_check 0 "knowledge-work appears in source list"
  fi
}

test_02b_knowledge_work_in_json() {
  local count
  count=$("$LOADOUT" list --json 2>/dev/null | jq '[.[] | select(.source == "knowledge-work")] | length')
  if [ "$count" -gt 0 ]; then
    _pass "knowledge-work present in JSON output ($count skills)"
    log_check 1 "knowledge-work has $count skills in --json output"
  else
    _fail "knowledge-work not in JSON output" "skills present" "0"
    log_check 0 "knowledge-work in --json output"
  fi
}

test_03_add_claude_official() {
  # claude-plugins-official marketplace.json uses object-format `source` entries
  # (e.g. {"source": "url", "url": "..."} and {"source": "git-subdir", ...})
  # alongside string-format entries (e.g. "./plugins/typescript-lsp").
  # Our marketplace parser must handle both formats.
  log_cmd "$LOADOUT" add https://github.com/anthropics/claude-plugins-official.git --source claude-official

  local json
  json=$("$LOADOUT" list --json 2>/dev/null)
  local skill_count
  skill_count=$(echo "$json" | jq '[.[] | select(.source == "claude-official")] | length')

  if [ "$skill_count" -gt 0 ]; then
    _pass "claude-official source added ($skill_count skills)"
    log_check 1 "claude-official has $skill_count skills"
  else
    _fail "claude-official has no skills (marketplace parser likely rejects object-format source entries)" \
      "skills present" "0 (check marketplace.json source field deserialization)"
    log_check 0 "claude-official skills detected"
  fi
}

test_04_add_financial_services() {
  log_cmd "$LOADOUT" add https://github.com/anthropics/financial-services-plugins.git --source financial-services
  local output
  output=$("$LOADOUT" list 2>/dev/null)
  if echo "$output" | grep -qF "financial-services"; then
    _pass "financial-services source added"
    log_check 1 "financial-services appears in source list"
  else
    _fail "financial-services not in list" "present" "not found"
    log_check 0 "financial-services appears in source list"
  fi
}

test_05_list_shows_all_sources() {
  local json_output
  json_output=$("$LOADOUT" list --json 2>/dev/null)
  log_cmd "$LOADOUT" list

  local found=0
  local total=0
  for name in anthropic-skills knowledge-work claude-official financial-services; do
    total=$((total + 1))
    local count
    count=$(echo "$json_output" | jq --arg src "$name" '[.[] | select(.source == $src)] | length')
    if [ "$count" -gt 0 ]; then
      log_check 1 "$name in list ($count skills)"
      found=$((found + 1))
    else
      log_check 0 "$name in list"
    fi
  done

  if [ "$found" -ge 3 ]; then
    _pass "$found/$total sources appear in list"
  else
    _fail "too few sources in list" "at least 3" "$found"
  fi
}

test_06_update_all() {
  log_cmd "$LOADOUT" update
  assert_exit_code 0 "$LOADOUT" update
}

test_07_cache_dirs_exist() {
  local cache_base="$XDG_DATA_HOME/equip"

  for name in anthropic-skills knowledge-work claude-official financial-services; do
    local found=false
    if [ -d "$cache_base/external/$name" ]; then
      local count
      count=$(find "$cache_base/external/$name" -type f | wc -l | tr -d ' ')
      log_check 1 "cache dir $cache_base/external/$name/ exists ($count files)"
      found=true
    elif [ -d "$cache_base/sources/$name" ]; then
      local count
      count=$(find "$cache_base/sources/$name" -type f | wc -l | tr -d ' ')
      log_check 1 "cache dir $cache_base/sources/$name/ exists ($count files)"
      found=true
    fi

    if [ "$found" = true ]; then
      _pass "cache dir exists for $name"
    else
      _fail "cache dir missing for $name" "directory exists" "not found under $cache_base"
      log_check 0 "cache dir for $name exists"
    fi
  done
}
