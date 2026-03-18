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
  count=$("$LOADOUT" list --json "local:test-plugin/*" 2>/dev/null | jq 'length')

  if [ "$count" -eq 3 ]; then
    _pass "list 'local:test-plugin/*' returns 3 skills"
  else
    _fail "source-qualified glob count wrong" "3" "$count"
  fi
}

test_glob_list_wrong_source() {
  setup_source_and_agents
  local output
  output=$("$LOADOUT" list --json "wrong-source:test-plugin/*" 2>&1)
  local exit_code=$?

  if [ "$exit_code" -ne 0 ] && echo "$output" | grep -qiE "no skill|no match"; then
    _pass "glob with wrong source returns a no-match error"
  else
    _fail "wrong source should return a no-match error" "non-zero exit with no-match message" "$output"
  fi
}

test_glob_list_partial_name() {
  setup_source_and_agents
  # "test-plugin/ex*" should match "explore" — single result returns detail object
  local output
  output=$("$LOADOUT" list --json "test-plugin/ex*" 2>/dev/null)
  if echo "$output" | jq -e '.name == "explore"' >/dev/null 2>&1; then
    _pass "list 'test-plugin/ex*' matches explore"
  elif echo "$output" | jq -e '.[0].name == "explore"' >/dev/null 2>&1; then
    _pass "list 'test-plugin/ex*' matches explore (array)"
  else
    _fail "partial glob did not match explore" "explore" "$output"
  fi
}

test_glob_list_question_mark() {
  setup_source_and_agents
  # "test-plugin/appl?" should match "apply" — single result returns detail object
  local output
  output=$("$LOADOUT" list --json "test-plugin/appl?" 2>/dev/null)
  if echo "$output" | jq -e '.name == "apply"' >/dev/null 2>&1; then
    _pass "list 'test-plugin/appl?' matches apply"
  elif echo "$output" | jq -e '.[0].name == "apply"' >/dev/null 2>&1; then
    _pass "list 'test-plugin/appl?' matches apply (array)"
  else
    _fail "? glob did not match apply" "apply" "$output"
  fi
}

test_glob_list_no_match() {
  setup_source_and_agents
  local output
  output=$("$LOADOUT" list --json "nonexistent/*" 2>&1)
  local exit_code=$?

  if [ "$exit_code" -ne 0 ] && echo "$output" | grep -qiE "no skill|no match"; then
    _pass "nonexistent glob returns a no-match error"
  else
    _fail "nonexistent glob should return a no-match error" "non-zero exit with no-match message" "$output"
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
  # "*plor*" should match "explore" — single result returns detail object
  local output
  output=$("$LOADOUT" list --json "*plor*" 2>/dev/null)
  if echo "$output" | jq -e '.name == "explore"' >/dev/null 2>&1; then
    _pass "freeform '*plor*' matches explore"
  elif echo "$output" | jq -e '.[0].name == "explore"' >/dev/null 2>&1; then
    _pass "freeform '*plor*' matches explore (array)"
  else
    _fail "freeform '*plor*' did not match explore" "explore" "$output"
  fi
}

test_glob_freeform_skill_name() {
  setup_source_and_agents
  # "verif*" should match "verify" — single result returns detail object
  local output
  output=$("$LOADOUT" list --json "verif*" 2>/dev/null)
  if echo "$output" | jq -e '.name == "verify"' >/dev/null 2>&1; then
    _pass "freeform 'verif*' matches verify"
  elif echo "$output" | jq -e '.[0].name == "verify"' >/dev/null 2>&1; then
    _pass "freeform 'verif*' matches verify (array)"
  else
    _fail "freeform 'verif*' did not match verify" "verify" "$output"
  fi
}

test_glob_freeform_no_match() {
  setup_source_and_agents
  local output
  output=$("$LOADOUT" list --json "zzzzz*" 2>&1)
  local exit_code=$?

  if [ "$exit_code" -ne 0 ] && echo "$output" | grep -qiE "no skill|no match"; then
    _pass "freeform 'zzzzz*' returns a no-match error"
  else
    _fail "freeform should return a no-match error" "non-zero exit with no-match message" "$output"
  fi
}

test_glob_apply_wildcard() {
  setup_source_and_agents
  "$LOADOUT" @test-claude "test-plugin/*" -f >/dev/null 2>&1
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
  "$LOADOUT" @test-claude "test-plugin/v*" -f >/dev/null 2>&1

  assert_file_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
  assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
}

test_glob_bundle_add() {
  setup_source_and_agents
  "$LOADOUT" kit create glob-b >/dev/null 2>&1
  "$LOADOUT" kit add glob-b "test-plugin/e*" >/dev/null 2>&1
  local exit_code=$?

  if [ "$exit_code" -eq 0 ]; then
    "$LOADOUT" @test-claude +glob-b -f >/dev/null 2>&1
    assert_file_exists "$TARGET_CLAUDE/skills/explore/SKILL.md"
    assert_file_not_exists "$TARGET_CLAUDE/skills/apply/SKILL.md"
    assert_file_not_exists "$TARGET_CLAUDE/skills/verify/SKILL.md"
    _pass "bundle add with glob expanded to explore only"
  else
    _fail "bundle add with glob failed" "exit 0" "exit $exit_code"
  fi
}
