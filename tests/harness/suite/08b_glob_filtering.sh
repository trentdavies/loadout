#!/usr/bin/env bash
# Suite 08b: Glob Filtering — verify glob patterns in list and apply
# Uses the test-plugin fixture (skills: apply, explore, verify).

test_glob_list_wildcard_all() {
  setup_source_and_agents
  local all_count glob_count
  all_count=$("$LOADOUT" list --json 2>/dev/null | jq 'length')
  glob_count=$("$LOADOUT" list --json "*/*" 2>/dev/null | jq 'length')

  if [ "$glob_count" -eq "$all_count" ] && [ "$glob_count" -gt 0 ]; then
    _pass "list '*/*' returns all $all_count skills"
  else
    _fail "list '*/*' count mismatch" "$all_count" "$glob_count"
  fi
}

test_glob_list_by_plugin() {
  setup_source_and_agents
  local count
  count=$("$LOADOUT" list --json "test-plugin/*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 3 ]; then
    _pass "list 'test-plugin/*' returns 3 skills"
  else
    _fail "list 'test-plugin/*' count wrong" "3" "$count"
  fi
}

test_glob_list_by_source_qualified() {
  setup_source_and_agents
  local count
  count=$("$LOADOUT" list --json "test-plugin:test-plugin/*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 3 ]; then
    _pass "list 'test-plugin:test-plugin/*' returns 3 skills"
  else
    _fail "source-qualified glob count wrong" "3" "$count"
  fi
}

test_glob_list_wrong_source() {
  setup_source_and_agents
  local count
  count=$("$LOADOUT" list --json "wrong-source:test-plugin/*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 0 ]; then
    _pass "glob with wrong source returns 0"
  else
    _fail "wrong source should return 0" "0" "$count"
  fi
}

test_glob_list_partial_name() {
  setup_source_and_agents
  # "test-plugin/ex*" should match "explore"
  local count
  count=$("$LOADOUT" list --json "test-plugin/ex*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 1 ]; then
    _pass "list 'test-plugin/ex*' matches explore"
  else
    _fail "partial glob count wrong" "1" "$count"
  fi
}

test_glob_list_question_mark() {
  setup_source_and_agents
  # "test-plugin/appl?" should match "apply"
  local count
  count=$("$LOADOUT" list --json "test-plugin/appl?" 2>/dev/null | jq 'length')

  if [ "$count" -eq 1 ]; then
    _pass "list 'test-plugin/appl?' matches apply"
  else
    _fail "? glob count wrong" "1" "$count"
  fi
}

test_glob_list_no_match() {
  setup_source_and_agents
  local count
  count=$("$LOADOUT" list --json "nonexistent/*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 0 ]; then
    _pass "nonexistent glob returns empty"
  else
    _fail "nonexistent glob should return 0" "0" "$count"
  fi
}

test_glob_freeform_source_prefix() {
  setup_source_and_agents
  # "test-*" has no / or : — should match all skills from test-plugin source
  local count
  count=$("$LOADOUT" list --json "test-*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 3 ]; then
    _pass "freeform 'test-*' matches all 3 skills"
  else
    _fail "freeform 'test-*' count wrong" "3" "$count"
  fi
}

test_glob_freeform_substring() {
  setup_source_and_agents
  # "*plor*" should match "explore" (substring of skill name)
  local count
  count=$("$LOADOUT" list --json "*plor*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 1 ]; then
    _pass "freeform '*plor*' matches explore"
  else
    _fail "freeform '*plor*' count wrong" "1" "$count"
  fi
}

test_glob_freeform_skill_name() {
  setup_source_and_agents
  # "verif*" should match "verify"
  local count
  count=$("$LOADOUT" list --json "verif*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 1 ]; then
    _pass "freeform 'verif*' matches verify"
  else
    _fail "freeform 'verif*' count wrong" "1" "$count"
  fi
}

test_glob_freeform_no_match() {
  setup_source_and_agents
  local count
  count=$("$LOADOUT" list --json "zzzzz*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 0 ]; then
    _pass "freeform 'zzzzz*' returns empty"
  else
    _fail "freeform should return 0" "0" "$count"
  fi
}

test_glob_apply_wildcard() {
  setup_source_and_agents
  "$LOADOUT" apply --force --skill "test-plugin/*" --agent test-claude >/dev/null 2>&1
  local exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    # All 3 skills should be installed
    assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
    assert_file_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
    assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
    _pass "glob apply installed all 3 skills"
  else
    _fail "glob apply failed" "exit 0" "exit $exit_code"
  fi
}

test_glob_apply_partial() {
  setup_source_and_agents
  # "test-plugin/v*" should only install verify
  "$LOADOUT" apply --force --skill "test-plugin/v*" --agent test-claude >/dev/null 2>&1

  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_glob_bundle_add() {
  setup_source_and_agents
  "$LOADOUT" bundle create glob-b >/dev/null 2>&1
  "$LOADOUT" bundle add glob-b "test-plugin/e*" >/dev/null 2>&1
  local exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    "$LOADOUT" apply --force --bundle glob-b --agent test-claude >/dev/null 2>&1
    assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
    assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
    assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
    _pass "bundle add with glob expanded to explore only"
  else
    _fail "bundle add with glob failed" "exit 0" "exit $exit_code"
  fi
}
