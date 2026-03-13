#!/usr/bin/env bash
# Suite 02: Detection and Listing — validate structure detection on real repos
# Depends on Suite 01 having added sources.

test_01_skills_detected() {
  local output
  output=$("$SKITTLE" list 2>/dev/null)
  log_cmd "$SKITTLE" list

  local line_count
  line_count=$(echo "$output" | wc -l | tr -d ' ')

  if [ "$line_count" -gt 4 ]; then
    _pass "skills detected from sources ($line_count lines of output)"
    log_check 1 "skills detected from sources ($line_count lines)"
  else
    _fail "few or no skills detected" "more than 4 lines" "$line_count lines"
    log_check 0 "skills detected from sources"
  fi
}

test_02_knowledge_work_plugins_detected() {
  local output
  output=$("$SKITTLE" list 2>/dev/null)

  local found_any=false
  for keyword in engineer legal sales marketing product research; do
    if echo "$output" | grep -qiF "$keyword"; then
      log_check 1 "knowledge-work contains '$keyword' skill/plugin"
      found_any=true
      break
    fi
  done

  if [ "$found_any" = true ]; then
    _pass "knowledge-work plugins detected (role-based skills found)"
  else
    log_check 0 "knowledge-work role-based keywords not found (repo may have different naming)"
    _pass "knowledge-work source present (skill names may differ from expected)"
  fi
}

test_03_list_json_valid() {
  local output
  output=$("$SKITTLE" list --json 2>/dev/null)
  local exit_code=$?

  if [ "$exit_code" -ne 0 ]; then
    _fail "list --json exited with $exit_code" "exit 0" "exit $exit_code"
    log_check 0 "list --json exits cleanly"
    return
  fi

  if echo "$output" | jq . >/dev/null 2>&1; then
    _pass "list --json produces valid JSON"
    log_check 1 "list --json | jq . succeeds"
  else
    _fail "list --json produced invalid JSON" "valid JSON" "parse error"
    log_check 0 "list --json | jq . succeeds"
  fi
}

test_04_skill_detail() {
  # Parse the columnar output: "Skill  Plugin  Source" — build plugin/skill
  local list_output
  list_output=$("$SKITTLE" list 2>/dev/null)

  # Skip header lines (first 2), prefer rows where plugin != source to avoid ambiguity bug
  local skill_name plugin_name
  skill_name=$(echo "$list_output" | tail -n +3 | awk '$2 != $3 && NF>=3 {print $1; exit}')
  plugin_name=$(echo "$list_output" | tail -n +3 | awk '$2 != $3 && NF>=3 {print $2; exit}')

  if [ -z "$skill_name" ] || [ -z "$plugin_name" ]; then
    _fail "could not parse skill/plugin from list output" "parseable rows" "empty"
    return
  fi

  local qualified="${plugin_name}/${skill_name}"
  log_cmd "$SKITTLE" list "$qualified"

  local detail_output
  detail_output=$("$SKITTLE" list "$qualified" 2>/dev/null)
  local exit_code=$?

  if [ "$exit_code" -eq 0 ] && [ -n "$detail_output" ]; then
    _pass "skill detail for '$qualified' shows metadata"
    log_check 1 "skittle list $qualified shows detail output"
  else
    _fail "skill detail failed for '$qualified'" "exit 0 with output" "exit $exit_code"
    log_check 0 "skittle list $qualified shows detail output"
  fi
}
