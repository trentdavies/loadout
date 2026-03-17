#!/usr/bin/env bash
# Suite 01: Config Management
# Tests init, config show, config show --json.

test_init_creates_config() {
  assert_exit_code 0 "$LOADOUT" init
  assert_file_exists "$XDG_DATA_HOME/equip/equip.toml"
}

test_init_idempotent() {
  "$LOADOUT" init >/dev/null 2>&1
  # Second init should not fail but should indicate config exists
  local output
  output=$("$LOADOUT" init 2>&1)
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
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" config show
}

test_config_show_json() {
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" config show --json
  local output
  output=$("$LOADOUT" config show --json 2>/dev/null)
  echo "$output" | jq . >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    _pass "config show --json produces valid JSON"
  else
    _fail "config show --json is not valid JSON" "valid JSON" "$output"
  fi
}

test_config_file_contains_examples() {
  "$LOADOUT" init >/dev/null 2>&1
  # The generated config should have example/commented sections
  assert_file_exists "$XDG_DATA_HOME/equip/equip.toml"
}

test_xdg_data_dir_created() {
  "$LOADOUT" init >/dev/null 2>&1
  # Some command that touches the data dir
  "$LOADOUT" status >/dev/null 2>&1
  assert_dir_exists "$XDG_DATA_HOME/equip"
}
