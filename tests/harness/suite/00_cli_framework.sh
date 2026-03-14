#!/usr/bin/env bash
# Suite 00: CLI Framework
# Tests help flags, unknown commands, exit codes, global flags.

test_help_long_flag() {
  assert_exit_code 0 "$SKITTLE" --help
  assert_stdout_contains "apply" "$SKITTLE" --help
  assert_stdout_contains "uninstall" "$SKITTLE" --help
  assert_stdout_contains "add" "$SKITTLE" --help
  assert_stdout_contains "remove" "$SKITTLE" --help
  assert_stdout_contains "update" "$SKITTLE" --help
  assert_stdout_contains "list" "$SKITTLE" --help
  assert_stdout_contains "target" "$SKITTLE" --help
  assert_stdout_contains "bundle" "$SKITTLE" --help
  assert_stdout_contains "status" "$SKITTLE" --help
  assert_stdout_contains "config" "$SKITTLE" --help
  assert_stdout_contains "init" "$SKITTLE" --help
}

test_help_short_flag() {
  assert_exit_code 0 "$SKITTLE" -h
}

test_help_subcommand() {
  assert_exit_code 0 "$SKITTLE" help
}

test_unknown_command_errors() {
  assert_exit_code 2 "$SKITTLE" foobar
  assert_stderr_contains "error" "$SKITTLE" foobar
}

test_target_subcommand_help() {
  assert_exit_code 0 "$SKITTLE" target --help
  assert_stdout_contains "add" "$SKITTLE" target --help
  assert_stdout_contains "remove" "$SKITTLE" target --help
  assert_stdout_contains "list" "$SKITTLE" target --help
  assert_stdout_contains "show" "$SKITTLE" target --help
  assert_stdout_contains "detect" "$SKITTLE" target --help
}

test_bundle_subcommand_help() {
  assert_exit_code 0 "$SKITTLE" bundle --help
  assert_stdout_contains "create" "$SKITTLE" bundle --help
  assert_stdout_contains "delete" "$SKITTLE" bundle --help
  assert_stdout_contains "list" "$SKITTLE" bundle --help
  assert_stdout_contains "show" "$SKITTLE" bundle --help
  assert_stdout_contains "add" "$SKITTLE" bundle --help
  assert_stdout_contains "drop" "$SKITTLE" bundle --help
  assert_stdout_contains "swap" "$SKITTLE" bundle --help
}

test_config_subcommand_help() {
  assert_exit_code 0 "$SKITTLE" config --help
  assert_stdout_contains "show" "$SKITTLE" config --help
  assert_stdout_contains "edit" "$SKITTLE" config --help
}

test_apply_no_flags_errors() {
  # apply with no flags should show help and exit non-zero
  assert_exit_code 2 "$SKITTLE" apply
}

test_uninstall_no_flags_errors() {
  # uninstall with no flags should show help and exit non-zero
  assert_exit_code 2 "$SKITTLE" uninstall
}

test_global_json_flag() {
  # status --json should produce valid JSON
  "$SKITTLE" init >/dev/null 2>&1
  local output
  output=$("$SKITTLE" status --json 2>/dev/null)
  assert_exit_code 0 "$SKITTLE" status --json
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
  setup_source_and_targets
  assert_exit_code 0 "$SKITTLE" apply --all -n
  # Verify no skills were actually installed
  assert_file_not_exists "$TARGET_CLAUDE/skills"
}

test_global_quiet_flag() {
  # -q should suppress non-error output
  "$SKITTLE" init >/dev/null 2>&1
  local output
  output=$("$SKITTLE" status -q 2>/dev/null)
  if [ -z "$output" ]; then
    _pass "quiet flag suppresses output"
  else
    _fail "quiet flag did not suppress output" "empty" "$output"
  fi
}

test_global_verbose_flag() {
  # -v should be accepted without error
  "$SKITTLE" init >/dev/null 2>&1
  assert_exit_code 0 "$SKITTLE" status -v
}
