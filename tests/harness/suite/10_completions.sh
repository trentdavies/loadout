#!/usr/bin/env bash
# Suite 10: Bash Completion
# Tests the completion script logic: @agent, +kit, progressive identity, globs.
#
# Strategy: source the completion script, stub `equip _complete` with known
# output, then simulate readline state and call the completion function.

# ---------------------------------------------------------------------------
# Completion test helpers
# ---------------------------------------------------------------------------

# Source bash-completion library (provides _init_completion, __ltrim_colon_completions)
_completion_loaded=false
for f in /usr/share/bash-completion/bash_completion /etc/bash_completion; do
    if [ -f "$f" ]; then
        source "$f"
        _completion_loaded=true
        break
    fi
done

if ! $_completion_loaded; then
    test_completion_skipped() {
        _skip "bash-completion not installed — skipping completion tests"
    }
    return 0 2>/dev/null || exit 0
fi

# Generate and source the completion script
_comp_script=$("$LOADOUT" completions bash 2>/dev/null)
if [ -z "$_comp_script" ]; then
    test_completion_skipped() {
        _skip "equip completions bash produced no output"
    }
    return 0 2>/dev/null || exit 0
fi
eval "$_comp_script"

# Stub equip to intercept `_complete` calls with known test data
_original_equip="$LOADOUT"
equip() {
    if [ "$1" = "_complete" ]; then
        case "$2" in
            sources)
                printf '%s\n' "anthropic-skills" "knowledge-work" "local-plugins"
                ;;
            agents)
                printf '%s\n' "claude" "codex" "cursor"
                ;;
            kits)
                printf '%s\n' "developer" "writing" "devops"
                ;;
            plugins)
                printf '%s\n' "anthropic-skills:document-skills" "anthropic-skills:example-skills" "knowledge-work:productivity" "local-plugins:tools"
                ;;
            skills)
                printf '%s\n' \
                    "anthropic-skills:document-skills/docx" \
                    "anthropic-skills:document-skills/pdf" \
                    "anthropic-skills:document-skills/pptx" \
                    "anthropic-skills:example-skills/frontend-design" \
                    "knowledge-work:productivity/memory-management" \
                    "knowledge-work:productivity/start" \
                    "knowledge-work:productivity/task-management" \
                    "local-plugins:tools/debug-helper"
                ;;
        esac
        return 0
    fi
    "$_original_equip" "$@"
}

# Simulate completion: set up COMP_WORDS, COMP_CWORD, COMP_LINE, COMP_POINT,
# then call _equip and capture COMPREPLY.
#
# Usage: _simulate_completion "equip" "@cl"
#   → sets COMP_WORDS=("equip" "@cl"), COMP_CWORD=1, calls _equip
_simulate_completion() {
    COMPREPLY=()
    COMP_WORDS=("$@")
    COMP_CWORD=$(( ${#COMP_WORDS[@]} - 1 ))
    COMP_LINE="${COMP_WORDS[*]}"
    COMP_POINT=${#COMP_LINE}
    # compopt is a no-op outside real readline — suppress its warnings
    _equip 2>/dev/null
}

# Assert COMPREPLY contains a specific value
_assert_compreply_contains() {
    local expected="$1"
    local msg="$2"
    local found=false
    for reply in "${COMPREPLY[@]}"; do
        if [ "$reply" = "$expected" ]; then
            found=true
            break
        fi
    done
    if $found; then
        _pass "$msg"
    else
        _fail "$msg" "'$expected' in COMPREPLY" "COMPREPLY=(${COMPREPLY[*]})"
    fi
}

# Assert COMPREPLY does NOT contain a value
_assert_compreply_not_contains() {
    local unexpected="$1"
    local msg="$2"
    local found=false
    for reply in "${COMPREPLY[@]}"; do
        if [ "$reply" = "$unexpected" ]; then
            found=true
            break
        fi
    done
    if ! $found; then
        _pass "$msg"
    else
        _fail "$msg" "'$unexpected' NOT in COMPREPLY" "COMPREPLY=(${COMPREPLY[*]})"
    fi
}

# Assert COMPREPLY has exactly N entries
_assert_compreply_count() {
    local expected="$1"
    local msg="$2"
    local actual=${#COMPREPLY[@]}
    if [ "$actual" -eq "$expected" ]; then
        _pass "$msg"
    else
        _fail "$msg" "$expected entries" "$actual entries: (${COMPREPLY[*]})"
    fi
}

# Assert COMPREPLY is exactly the given values (order-independent)
_assert_compreply_eq() {
    local msg="$1"; shift
    local expected=("$@")
    local actual_sorted=$(printf '%s\n' "${COMPREPLY[@]}" | sort)
    local expected_sorted=$(printf '%s\n' "${expected[@]}" | sort)
    if [ "$actual_sorted" = "$expected_sorted" ]; then
        _pass "$msg"
    else
        _fail "$msg" "(${expected[*]})" "(${COMPREPLY[*]})"
    fi
}

# ---------------------------------------------------------------------------
# Tests: @agent completion
# ---------------------------------------------------------------------------

test_01_at_empty_lists_all_agents() {
    _simulate_completion "equip" "@"
    _assert_compreply_eq "@<TAB> lists all agents" "@claude" "@codex" "@cursor"
}

test_02_at_prefix_filters_agents() {
    _simulate_completion "equip" "@cl"
    _assert_compreply_eq "@cl<TAB> matches claude only" "@claude"
}

test_03_at_prefix_multiple_matches() {
    _simulate_completion "equip" "@c"
    _assert_compreply_eq "@c<TAB> matches claude, codex, cursor" "@claude" "@codex" "@cursor"
}

test_04_at_no_match() {
    _simulate_completion "equip" "@zzz"
    _assert_compreply_count 0 "@zzz<TAB> matches nothing"
}

# ---------------------------------------------------------------------------
# Tests: +kit completion
# ---------------------------------------------------------------------------

test_05_plus_empty_lists_all_kits() {
    _simulate_completion "equip" "+"
    _assert_compreply_eq "+<TAB> lists all kits" "+developer" "+writing" "+devops"
}

test_06_plus_prefix_filters_kits() {
    _simulate_completion "equip" "+dev"
    _assert_compreply_eq "+dev<TAB> matches developer and devops" "+developer" "+devops"
}

test_07_plus_unique_match() {
    _simulate_completion "equip" "+wr"
    _assert_compreply_eq "+wr<TAB> matches writing only" "+writing"
}

# ---------------------------------------------------------------------------
# Tests: progressive identity completion (source:plugin/skill)
#
# __ltrim_colon_completions strips everything up to and including the last
# colon in $cur from each COMPREPLY entry. This is what readline needs.
# So tests check the post-trim values.
# ---------------------------------------------------------------------------

test_10_identity_level1_empty() {
    # After @agent, empty string should list source names
    _simulate_completion "equip" "@claude" ""
    _assert_compreply_contains "anthropic-skills" "empty arg after @agent lists sources"
    _assert_compreply_contains "knowledge-work" "empty arg lists knowledge-work"
    _assert_compreply_contains "local-plugins" "empty arg lists local-plugins"
}

test_11_identity_level1_prefix() {
    _simulate_completion "equip" "@claude" "anth"
    _assert_compreply_count 1 "anth<TAB> matches one source"
    # Unique match appends ":"
    _assert_compreply_contains "anthropic-skills:" "anth<TAB> completes to anthropic-skills:"
}

test_12_identity_level1_unique_appends_colon() {
    _simulate_completion "equip" "@claude" "local"
    _assert_compreply_count 1 "local<TAB> matches one source"
    _assert_compreply_contains "local-plugins:" "unique source appends colon"
}

test_13_identity_level2_empty_plugin() {
    # cur="anthropic-skills:" — __ltrim_colon_completions strips "anthropic-skills:"
    # from COMPREPLY, leaving just the plugin names (with trailing / if unique)
    _simulate_completion "equip" "@claude" "anthropic-skills:"
    _assert_compreply_contains "document-skills" "source:<TAB> lists document-skills (trimmed)"
    _assert_compreply_contains "example-skills" "source:<TAB> lists example-skills (trimmed)"
}

test_14_identity_level2_prefix() {
    # cur="anthropic-skills:doc" → after ltrim: "document-skills/"
    _simulate_completion "equip" "@claude" "anthropic-skills:doc"
    _assert_compreply_count 1 "source:doc<TAB> matches one plugin"
    _assert_compreply_contains "document-skills/" "plugin match appends slash (trimmed)"
}

test_15_identity_level2_unique_appends_slash() {
    _simulate_completion "equip" "@claude" "knowledge-work:prod"
    _assert_compreply_count 1 "knowledge-work:prod<TAB> matches one plugin"
    _assert_compreply_contains "productivity/" "unique plugin appends slash (trimmed)"
}

test_16_identity_level3_lists_skills() {
    # cur="anthropic-skills:document-skills/" → after ltrim on last colon:
    # "anthropic-skills:" is stripped, leaving "document-skills/docx" etc.
    _simulate_completion "equip" "@claude" "anthropic-skills:document-skills/"
    _assert_compreply_contains "document-skills/docx" "level3 lists docx (trimmed)"
    _assert_compreply_contains "document-skills/pdf" "level3 lists pdf (trimmed)"
    _assert_compreply_contains "document-skills/pptx" "level3 lists pptx (trimmed)"
}

test_17_identity_level3_prefix() {
    _simulate_completion "equip" "@claude" "anthropic-skills:document-skills/do"
    _assert_compreply_count 1 "level3 do<TAB> matches docx only"
    _assert_compreply_contains "document-skills/docx" "level3 completes to docx (trimmed)"
}

test_18_identity_full_sequence() {
    # Simulate the progressive completion sequence

    # Step 1: "anth" → "anthropic-skills:"
    _simulate_completion "equip" "@claude" "anth"
    _assert_compreply_contains "anthropic-skills:" "step1: anth → anthropic-skills:"

    # Step 2: "anthropic-skills:doc" → after ltrim: "document-skills/"
    _simulate_completion "equip" "@claude" "anthropic-skills:doc"
    _assert_compreply_contains "document-skills/" "step2: doc → document-skills/"

    # Step 3: "anthropic-skills:document-skills/do" → after ltrim: "document-skills/docx"
    _simulate_completion "equip" "@claude" "anthropic-skills:document-skills/do"
    _assert_compreply_contains "document-skills/docx" "step3: do → docx"
}

# ---------------------------------------------------------------------------
# Tests: glob suppression
# ---------------------------------------------------------------------------

test_20_glob_star_suppresses_completion() {
    _simulate_completion "equip" "@claude" "doc*"
    _assert_compreply_count 0 "doc*<TAB> suppressed (contains *)"
}

test_21_glob_question_suppresses_completion() {
    _simulate_completion "equip" "@claude" "doc?"
    _assert_compreply_count 0 "doc?<TAB> suppressed (contains ?)"
}

test_22_glob_bracket_suppresses_completion() {
    _simulate_completion "equip" "@claude" "doc[x"
    _assert_compreply_count 0 "doc[x<TAB> suppressed (contains [)"
}

# ---------------------------------------------------------------------------
# Tests: shorthand context — flags after @agent/+kit
# ---------------------------------------------------------------------------

test_30_shorthand_flags_after_agent() {
    _simulate_completion "equip" "@claude" "-"
    _assert_compreply_contains "-f" "flags available after @agent"
    _assert_compreply_contains "-r" "--remove flag available"
    _assert_compreply_contains "-s" "--save flag available"
    _assert_compreply_contains "-i" "--interactive flag available"
}

test_31_shorthand_flags_after_kit() {
    _simulate_completion "equip" "+developer" "-"
    _assert_compreply_contains "-f" "flags available after +kit"
    _assert_compreply_contains "-r" "--remove flag available after +kit"
}

# ---------------------------------------------------------------------------
# Tests: top-level command completion (not shorthand)
# ---------------------------------------------------------------------------

test_40_toplevel_commands() {
    _simulate_completion "equip" ""
    _assert_compreply_contains "init" "top-level lists init"
    _assert_compreply_contains "add" "top-level lists add"
    _assert_compreply_contains "list" "top-level lists list"
    _assert_compreply_contains "status" "top-level lists status"
    _assert_compreply_contains "agent" "top-level lists agent"
    _assert_compreply_contains "kit" "top-level lists kit"
}

test_41_toplevel_prefix() {
    _simulate_completion "equip" "li"
    _assert_compreply_eq "li<TAB> matches list only" "list"
}

# ---------------------------------------------------------------------------
# Tests: list subcommand gets identity completion
# ---------------------------------------------------------------------------

test_50_list_completes_identities() {
    _simulate_completion "equip" "list" "anth"
    _assert_compreply_contains "anthropic-skills:" "list completes skill identities"
}

# ---------------------------------------------------------------------------
# Tests: agent subcommands (no equip/unequip)
# ---------------------------------------------------------------------------

test_60_agent_subcommands() {
    _simulate_completion "equip" "agent" ""
    _assert_compreply_contains "add" "agent lists add"
    _assert_compreply_contains "detect" "agent lists detect"
    _assert_compreply_contains "collect" "agent lists collect"
    _assert_compreply_not_contains "equip" "agent does NOT list equip subcommand"
    _assert_compreply_not_contains "unequip" "agent does NOT list unequip subcommand"
}

# ---------------------------------------------------------------------------
# Tests: add offers file completion (not just flags)
# ---------------------------------------------------------------------------

test_70_add_no_flag_does_not_error() {
    # With the _filedir fallback, completing a non-flag positional for 'add'
    # should not produce flag-only results. We can't test actual filesystem
    # results in the stub, but we verify it doesn't error and doesn't return
    # flag completions for a non-flag word.
    _simulate_completion "equip" "add" "some"
    # Should NOT contain flag entries since cur doesn't start with -
    _assert_compreply_not_contains "--source" "add positional does not suggest --source"
    _assert_compreply_not_contains "--plugin" "add positional does not suggest --plugin"
}

# ---------------------------------------------------------------------------
# Tests: agent collect completion — new flags and positional patterns
# ---------------------------------------------------------------------------

test_71_collect_flags_include_interactive() {
    _simulate_completion "equip" "agent" "collect" "-"
    _assert_compreply_contains "--agent" "collect flags include --agent"
    _assert_compreply_contains "--force" "collect flags include --force"
    _assert_compreply_contains "-f" "collect flags include -f"
    _assert_compreply_contains "--interactive" "collect flags include --interactive"
    _assert_compreply_contains "-i" "collect flags include -i"
    _assert_compreply_contains "--adopt" "collect flags include --adopt"
}

test_72_collect_flags_no_skill() {
    _simulate_completion "equip" "agent" "collect" "-"
    _assert_compreply_not_contains "--skill" "collect flags do NOT include --skill (removed)"
}

test_73_collect_positional_completes_skills() {
    _simulate_completion "equip" "agent" "collect" "--agent" "claude" "anth"
    _assert_compreply_contains "anthropic-skills:" "collect positional completes skill identities"
}
