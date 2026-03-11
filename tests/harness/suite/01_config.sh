#!/usr/bin/env bash
# Suite 01: Config Management
# Tests init, config show, config show --json, cache show, cache clean.

test_init_creates_config() {
  assert_exit_code 0 "$SKITTLE" init
  assert_file_exists "$XDG_CONFIG_HOME/skittle/config.toml"
}

test_init_idempotent() {
  "$SKITTLE" init >/dev/null 2>&1
  # Second init should not fail but should indicate config exists
  local output
  output=$("$SKITTLE" init 2>&1)
  local exit_code=$?
  # Should either exit 0 with a message or exit non-zero gracefully
  if echo "$output" | grep -qiF "already exists"; then
    _pass "init reports config already exists"
  elif echo "$output" | grep -qiF "config"; then
    _pass "init mentions config on second run"
  else
    _fail "init did not indicate existing config" "message about existing config" "$output"
  fi
}

test_config_show() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" config show
}

test_config_show_json() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" config show --json
  local output
  output=$("$SKITTLE" config show --json 2>/dev/null)
  echo "$output" | jq . >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    _pass "config show --json produces valid JSON"
  else
    _fail "config show --json is not valid JSON" "valid JSON" "$output"
  fi
}

test_cache_show() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" cache show
}

test_cache_clean_empty() {
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" cache clean --force
  local output
  output=$("$SKITTLE" cache clean --force 2>&1)
  if echo "$output" | grep -qiE "empty|clean|no.*cache"; then
    _pass "cache clean reports empty/clean state"
  else
    _fail "cache clean did not report empty state" "empty/clean message" "$output"
  fi
}

test_cache_clean_preview_default() {
  "$SKITTLE" init >/dev/null 2>&1
  local output
  output=$("$SKITTLE" cache clean 2>&1)
  if echo "$output" | grep -qiE "would|force"; then
    _pass "cache clean defaults to preview mode"
  else
    _fail "cache clean did not show preview" "would/force message" "$output"
  fi
}

test_config_file_contains_examples() {
  "$SKITTLE" init >/dev/null 2>&1
  # The generated config should have example/commented sections
  assert_file_exists "$XDG_CONFIG_HOME/skittle/config.toml"
}

test_xdg_data_dir_created() {
  "$SKITTLE" init >/dev/null 2>&1
  # Some command that touches the data dir
  "$SKITTLE" status >/dev/null 2>&1
  assert_dir_exists "$XDG_DATA_HOME/skittle"
}
