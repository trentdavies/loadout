#!/usr/bin/env bash
# Suite 00: CLI Framework
# Tests help flags, unknown commands, exit codes, global flags.

test_help_long_flag() {
  assert_exit_code 0 "$LOADOUT" --help
  assert_stdout_contains "apply" "$LOADOUT" --help
  assert_stdout_contains "uninstall" "$LOADOUT" --help
  assert_stdout_contains "add" "$LOADOUT" --help
  assert_stdout_contains "remove" "$LOADOUT" --help
  assert_stdout_contains "update" "$LOADOUT" --help
  assert_stdout_contains "list" "$LOADOUT" --help
  assert_stdout_contains "agent" "$LOADOUT" --help
  assert_stdout_contains "bundle" "$LOADOUT" --help
  assert_stdout_contains "status" "$LOADOUT" --help
  assert_stdout_contains "config" "$LOADOUT" --help
  assert_stdout_contains "init" "$LOADOUT" --help
}

test_help_short_flag() {
  assert_exit_code 0 "$LOADOUT" -h
}

test_help_subcommand() {
  assert_exit_code 0 "$LOADOUT" help
}

test_unknown_command_errors() {
  assert_exit_code 2 "$LOADOUT" foobar
  assert_stderr_contains "error" "$LOADOUT" foobar
}

test_agent_subcommand_help() {
  assert_exit_code 0 "$LOADOUT" agent --help
  assert_stdout_contains "add" "$LOADOUT" agent --help
  assert_stdout_contains "remove" "$LOADOUT" agent --help
  assert_stdout_contains "list" "$LOADOUT" agent --help
  assert_stdout_contains "show" "$LOADOUT" agent --help
  assert_stdout_contains "detect" "$LOADOUT" agent --help
}

test_bundle_subcommand_help() {
  assert_exit_code 0 "$LOADOUT" bundle --help
  assert_stdout_contains "create" "$LOADOUT" bundle --help
  assert_stdout_contains "delete" "$LOADOUT" bundle --help
  assert_stdout_contains "list" "$LOADOUT" bundle --help
  assert_stdout_contains "show" "$LOADOUT" bundle --help
  assert_stdout_contains "add" "$LOADOUT" bundle --help
  assert_stdout_contains "drop" "$LOADOUT" bundle --help
  assert_stdout_contains "activate" "$LOADOUT" bundle --help
  assert_stdout_contains "deactivate" "$LOADOUT" bundle --help
}

test_config_subcommand_help() {
  assert_exit_code 0 "$LOADOUT" config --help
  assert_stdout_contains "show" "$LOADOUT" config --help
  assert_stdout_contains "edit" "$LOADOUT" config --help
}

test_apply_no_flags_errors() {
  # apply with no flags should show help and exit non-zero
  assert_exit_code 2 "$LOADOUT" apply
}

test_uninstall_no_flags_errors() {
  # uninstall with no flags should show help and exit non-zero
  assert_exit_code 2 "$LOADOUT" uninstall
}

test_global_json_flag() {
  # status --json should produce valid JSON
  "$LOADOUT" init >/dev/null 2>&1
  local output
  output=$("$LOADOUT" status --json 2>/dev/null)
  assert_exit_code 0 "$LOADOUT" status --json
  # Validate it's parseable JSON
  echo "$output" | jq . >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    _pass "status --json produces valid JSON"
  else
    _fail "status --json does not produce valid JSON" "valid JSON" "$output"
  fi
}

test_global_dry_run_flag() {
  # apply --all -n should succeed without writing files
  setup_source_and_agents
  assert_exit_code 0 "$LOADOUT" apply --all -n
  # Verify no skills were actually installed
  assert_file_not_exists "$TARGET_CLAUDE/skills"
}

test_global_quiet_flag() {
  # -q should suppress non-error output
  "$LOADOUT" init >/dev/null 2>&1
  local output
  output=$("$LOADOUT" status -q 2>/dev/null)
  if [ -z "$output" ]; then
    _pass "quiet flag suppresses output"
  else
    _fail "quiet flag did not suppress output" "empty" "$output"
  fi
}

test_global_verbose_flag() {
  # -v should be accepted without error
  "$LOADOUT" init >/dev/null 2>&1
  assert_exit_code 0 "$LOADOUT" status -v
}
