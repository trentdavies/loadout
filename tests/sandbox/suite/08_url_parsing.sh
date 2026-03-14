#!/usr/bin/env bash
# Suite 08: URL Parsing — tree URLs, subpath detection, and default naming
#
# These tests define TARGET behavior for GitHub /tree/<ref>/<path> URLs.
# Tests that require tree URL support will fail until that feature is implemented.
# Each test adds a source with a unique --source name and cleans up after itself.

# ---------------------------------------------------------------------------
# Helper: add source, capture JSON, clean up
# ---------------------------------------------------------------------------
_add_and_list() {
  local url="$1"
  local source_name="$2"
  # Remove if leftover from a previous run
  "$LOADOUT" -q remove "$source_name" --force 2>/dev/null || true
  "$LOADOUT" -q add "$url" --source "$source_name" 2>&1
  local exit_code=$?
  if [ "$exit_code" -ne 0 ]; then
    echo "EXIT:$exit_code"
    return "$exit_code"
  fi
  "$LOADOUT" list --json 2>/dev/null
}

_cleanup_source() {
  local source_name="$1"
  "$LOADOUT" -q remove "$source_name" --force 2>/dev/null || true
}

# ---------------------------------------------------------------------------
# 1. anthropics/skills/tree/main — Marketplace at repo root
# ---------------------------------------------------------------------------
test_01_anthropics_skills_tree_main() {
  skip_if_no_network && return

  local source="anthropics-skills"
  local json
  json=$(_add_and_list "https://github.com/anthropics/skills/tree/main" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for anthropics/skills/tree/main" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count has_claude_api
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  has_claude_api=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "claude-api")] | length')

  if [ "$plugin_count" -ge 2 ]; then
    _pass "anthropics/skills marketplace has $plugin_count plugins"
  else
    _fail "anthropics/skills marketplace plugin count" ">=2" "$plugin_count"
  fi

  if [ "$skill_count" -ge 15 ]; then
    _pass "anthropics/skills has $skill_count skills"
  else
    _fail "anthropics/skills skill count" ">=15" "$skill_count"
  fi

  if [ "$has_claude_api" -ge 1 ]; then
    _pass "anthropics/skills contains claude-api skill"
  else
    _fail "anthropics/skills missing claude-api" "present" "not found"
  fi

  # Identity: marketplace.json defines plugin groups — plugin name != skill name
  # "document-skills" plugin contains xlsx, docx, pptx, pdf
  # "example-skills" plugin contains 12 skills
  # "claude-api" plugin contains claude-api
  local id_xlsx id_claude_api
  id_xlsx=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "xlsx")] | .[0].identity')
  id_claude_api=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "claude-api")] | .[0].identity')

  if [ "$id_xlsx" = "anthropics-skills:document-skills/xlsx" ]; then
    _pass "marketplace grouped identity correct: $id_xlsx"
  else
    _fail "marketplace grouped identity" "anthropics-skills:document-skills/xlsx" "$id_xlsx"
  fi

  if [ "$id_claude_api" = "anthropics-skills:claude-api/claude-api" ]; then
    _pass "marketplace single-skill plugin identity correct: $id_claude_api"
  else
    _fail "marketplace single-skill plugin identity" "anthropics-skills:claude-api/claude-api" "$id_claude_api"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 2. anthropics/skills/tree/main/skills — FlatSkills on subpath
# ---------------------------------------------------------------------------
test_02_anthropics_skills_flat() {
  skip_if_no_network && return

  local source="anthropics-flat"
  local json
  json=$(_add_and_list "https://github.com/anthropics/skills/tree/main/skills" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for anthropics/skills/tree/main/skills" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count has_claude_api
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  has_claude_api=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "claude-api")] | length')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "anthropics/skills/skills detected as FlatSkills (1 plugin)"
  else
    _fail "anthropics/skills/skills plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -ge 15 ]; then
    _pass "anthropics/skills/skills has $skill_count skills"
  else
    _fail "anthropics/skills/skills skill count" ">=15" "$skill_count"
  fi

  if [ "$has_claude_api" -ge 1 ]; then
    _pass "anthropics/skills/skills contains claude-api skill"
  else
    _fail "anthropics/skills/skills missing claude-api" "present" "not found"
  fi

  # Identity: FlatSkills plugin name = detection root dir = "skills"
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "claude-api")] | .[0].identity')
  if [ "$identity" = "anthropics-flat:skills/claude-api" ]; then
    _pass "flat claude-api identity correct: $identity"
  else
    _fail "flat claude-api identity" "anthropics-flat:skills/claude-api" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 3. anthropics/skills/tree/main/skills/claude-api — SingleSkillDir
# ---------------------------------------------------------------------------
test_03_anthropics_claude_api() {
  skip_if_no_network && return

  local source="a-claude-api"
  local json
  json=$(_add_and_list "https://github.com/anthropics/skills/tree/main/skills/claude-api" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for anthropics/skills/tree/main/skills/claude-api" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count skill_name
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  skill_name=$(echo "$json" | jq -r --arg s "$source" '[.[] | select(.source == $s)] | .[0].name')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "claude-api detected as SingleSkillDir (1 plugin)"
  else
    _fail "claude-api plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -eq 1 ]; then
    _pass "claude-api has exactly 1 skill"
  else
    _fail "claude-api skill count" "1" "$skill_count"
  fi

  if [ "$skill_name" = "claude-api" ]; then
    _pass "claude-api skill name is correct"
  else
    _fail "claude-api skill name" "claude-api" "$skill_name"
  fi

  # Identity: SingleSkillDir plugin name = source_name
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s)] | .[0].identity')
  if [ "$identity" = "a-claude-api:a-claude-api/claude-api" ]; then
    _pass "single claude-api identity correct: $identity"
  else
    _fail "single claude-api identity" "a-claude-api:a-claude-api/claude-api" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 4. cloudflare/skills — Marketplace (shorthand, no tree)
# ---------------------------------------------------------------------------
test_04_cloudflare_skills() {
  skip_if_no_network && return

  local source="cloudflare-skills"
  local json
  json=$(_add_and_list "cloudflare/skills" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for cloudflare/skills" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count has_cloudflare
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  has_cloudflare=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "cloudflare")] | length')

  if [ "$plugin_count" -ge 1 ]; then
    _pass "cloudflare/skills has $plugin_count plugins"
  else
    _fail "cloudflare/skills plugin count" ">=1" "$plugin_count"
  fi

  if [ "$skill_count" -ge 5 ]; then
    _pass "cloudflare/skills has $skill_count skills"
  else
    _fail "cloudflare/skills skill count" ">=5" "$skill_count"
  fi

  if [ "$has_cloudflare" -ge 1 ]; then
    _pass "cloudflare/skills contains cloudflare skill"
  else
    _fail "cloudflare/skills missing cloudflare" "present" "not found"
  fi

  # Identity: marketplace plugin name = dir name = cloudflare
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "cloudflare")] | .[0].identity')
  if [ "$identity" = "cloudflare-skills:cloudflare/cloudflare" ]; then
    _pass "cloudflare identity correct: $identity"
  else
    _fail "cloudflare identity" "cloudflare-skills:cloudflare/cloudflare" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 5. cloudflare/skills/tree/main — same as #4 with explicit ref
# ---------------------------------------------------------------------------
test_05_cloudflare_skills_tree_main() {
  skip_if_no_network && return

  local source="cf-skills-main"
  local json
  json=$(_add_and_list "https://github.com/cloudflare/skills/tree/main" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for cloudflare/skills/tree/main" "exit 0" "$json"
    return
  fi

  local skill_count has_cloudflare
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  has_cloudflare=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "cloudflare")] | length')

  if [ "$skill_count" -ge 5 ]; then
    _pass "cloudflare/skills/tree/main has $skill_count skills"
  else
    _fail "cloudflare/skills/tree/main skill count" ">=5" "$skill_count"
  fi

  if [ "$has_cloudflare" -ge 1 ]; then
    _pass "cloudflare/skills/tree/main contains cloudflare skill"
  else
    _fail "cloudflare/skills/tree/main missing cloudflare" "present" "not found"
  fi

  # Identity: same marketplace structure as #4 but different source name
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "cloudflare")] | .[0].identity')
  if [ "$identity" = "cf-skills-main:cloudflare/cloudflare" ]; then
    _pass "cf tree/main cloudflare identity correct: $identity"
  else
    _fail "cf tree/main cloudflare identity" "cf-skills-main:cloudflare/cloudflare" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 6. cloudflare/skills/tree/main/skills/cloudflare — SingleSkillDir
# ---------------------------------------------------------------------------
test_06_cloudflare_single_skill() {
  skip_if_no_network && return

  local source="cf-cloudflare"
  local json
  json=$(_add_and_list "https://github.com/cloudflare/skills/tree/main/skills/cloudflare" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for cloudflare/skills/tree/main/skills/cloudflare" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count skill_name
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  skill_name=$(echo "$json" | jq -r --arg s "$source" '[.[] | select(.source == $s)] | .[0].name')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "cloudflare single skill: 1 plugin"
  else
    _fail "cloudflare single skill plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -eq 1 ]; then
    _pass "cloudflare single skill: 1 skill"
  else
    _fail "cloudflare single skill count" "1" "$skill_count"
  fi

  if [ "$skill_name" = "cloudflare" ]; then
    _pass "cloudflare skill name is correct"
  else
    _fail "cloudflare skill name" "cloudflare" "$skill_name"
  fi

  # Identity: SingleSkillDir plugin name = source_name
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s)] | .[0].identity')
  if [ "$identity" = "cf-cloudflare:cf-cloudflare/cloudflare" ]; then
    _pass "cf single skill identity correct: $identity"
  else
    _fail "cf single skill identity" "cf-cloudflare:cf-cloudflare/cloudflare" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 7. openai/skills/tree/main — should find 0 skills (hidden dirs filtered)
# ---------------------------------------------------------------------------
test_07_openai_skills_no_skills() {
  skip_if_no_network && return

  local source="openai-skills"
  local output
  output=$("$LOADOUT" -q add "https://github.com/openai/skills/tree/main" --source "$source" 2>&1)
  local exit_code=$?

  # Either the add fails (error on no skills) or succeeds with 0 skills
  local skill_count=0
  if [ "$exit_code" -eq 0 ]; then
    skill_count=$("$LOADOUT" list --json 2>/dev/null | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  fi

  if [ "$skill_count" -eq 0 ]; then
    _pass "openai/skills/tree/main correctly finds 0 skills (hidden dirs filtered)"
  else
    _fail "openai/skills/tree/main should find 0 skills" "0" "$skill_count"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 8. openai/skills/tree/main/skills/.curated — FlatSkills (~34 skills)
# ---------------------------------------------------------------------------
test_08_openai_curated() {
  skip_if_no_network && return

  local source="openai-curated"
  local json
  json=$(_add_and_list "https://github.com/openai/skills/tree/main/skills/.curated" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for openai/skills/tree/main/skills/.curated" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count has_doc
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  has_doc=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "doc")] | length')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "openai .curated detected as FlatSkills (1 plugin)"
  else
    _fail "openai .curated plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -ge 20 ]; then
    _pass "openai .curated has $skill_count skills"
  else
    _fail "openai .curated skill count" ">=20" "$skill_count"
  fi

  if [ "$has_doc" -ge 1 ]; then
    _pass "openai .curated contains doc skill"
  else
    _fail "openai .curated missing doc" "present" "not found"
  fi

  # Identity: FlatSkills plugin name = dir name with leading dot stripped = "curated"
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "doc")] | .[0].identity')
  if [ "$identity" = "openai-curated:curated/doc" ]; then
    _pass "openai curated doc identity correct: $identity"
  else
    _fail "openai curated doc identity" "openai-curated:curated/doc" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 9. openai/skills/tree/main/skills/.curated/doc — SingleSkillDir
# ---------------------------------------------------------------------------
test_09_openai_doc_single() {
  skip_if_no_network && return

  local source="openai-doc"
  local json
  json=$(_add_and_list "https://github.com/openai/skills/tree/main/skills/.curated/doc" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for openai/skills/.curated/doc" "exit 0" "$json"
    return
  fi

  local skill_count skill_name
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  skill_name=$(echo "$json" | jq -r --arg s "$source" '[.[] | select(.source == $s)] | .[0].name')

  if [ "$skill_count" -eq 1 ]; then
    _pass "openai doc: exactly 1 skill"
  else
    _fail "openai doc skill count" "1" "$skill_count"
  fi

  if [ "$skill_name" = "doc" ]; then
    _pass "openai doc skill name is correct"
  else
    _fail "openai doc skill name" "doc" "$skill_name"
  fi

  # Identity: SingleSkillDir plugin name = source_name
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s)] | .[0].identity')
  if [ "$identity" = "openai-doc:openai-doc/doc" ]; then
    _pass "openai single doc identity correct: $identity"
  else
    _fail "openai single doc identity" "openai-doc:openai-doc/doc" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 10. oeftimie/vv-claude-harness/tree/main/claude/skills — FlatSkills
# ---------------------------------------------------------------------------
test_10_vv_harness() {
  skip_if_no_network && return

  local source="vv-harness"
  local json
  json=$(_add_and_list "https://github.com/oeftimie/vv-claude-harness/tree/main/claude/skills" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for vv-claude-harness/tree/main/claude/skills" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "vv-harness detected as FlatSkills (1 plugin)"
  else
    _fail "vv-harness plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -eq 2 ]; then
    _pass "vv-harness has exactly 2 skills"
  else
    _fail "vv-harness skill count" "2" "$skill_count"
  fi

  # Identity: FlatSkills plugin name = detection root dir = "skills"
  # All identities should match vv-harness:skills/<skill>
  local bad_identities
  bad_identities=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s) | select(.identity | startswith("vv-harness:skills/") | not)] | length')
  if [ "$bad_identities" -eq 0 ]; then
    _pass "vv-harness identities all match vv-harness:skills/*"
  else
    local sample
    sample=$(echo "$json" | jq -r --arg s "$source" '[.[] | select(.source == $s)] | .[0].identity')
    _fail "vv-harness identity format" "vv-harness:skills/*" "$sample"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 11. trentdavies/writing-assistant — SinglePlugin (no tree URL)
# ---------------------------------------------------------------------------
test_11_writing_assistant() {
  skip_if_no_network && return

  local source="writing-asst"
  local json
  json=$(_add_and_list "trentdavies/writing-assistant" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for trentdavies/writing-assistant" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count has_content_writer
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')
  has_content_writer=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s and .name == "content-writer")] | length')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "writing-assistant: SinglePlugin (1 plugin)"
  else
    _fail "writing-assistant plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -eq 7 ]; then
    _pass "writing-assistant has 7 skills"
  else
    _fail "writing-assistant skill count" "7" "$skill_count"
  fi

  if [ "$has_content_writer" -ge 1 ]; then
    _pass "writing-assistant contains content-writer skill"
  else
    _fail "writing-assistant missing content-writer" "present" "not found"
  fi

  # Identity: SinglePlugin plugin name from plugin.json = "writing-assistant"
  local identity
  identity=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s and .name == "content-writer")] | .[0].identity')
  if [ "$identity" = "writing-asst:writing-assistant/content-writer" ]; then
    _pass "writing-assistant content-writer identity correct: $identity"
  else
    _fail "writing-assistant content-writer identity" "writing-asst:writing-assistant/content-writer" "$identity"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 12. anthropics/knowledge-work-plugins — Marketplace (no tree URL)
# ---------------------------------------------------------------------------
test_12_knowledge_work_plugins() {
  skip_if_no_network && return

  local source="kw-plugins"
  local json
  json=$(_add_and_list "anthropics/knowledge-work-plugins" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for anthropics/knowledge-work-plugins" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')

  if [ "$plugin_count" -ge 10 ]; then
    _pass "knowledge-work-plugins has $plugin_count plugins"
  else
    _fail "knowledge-work-plugins plugin count" ">=10" "$plugin_count"
  fi

  if [ "$skill_count" -ge 20 ]; then
    _pass "knowledge-work-plugins has $skill_count skills"
  else
    _fail "knowledge-work-plugins skill count" ">=20" "$skill_count"
  fi

  # Identity: Marketplace — all identities should match kw-plugins:<plugin>/<skill>
  local bad_identities
  bad_identities=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s) | select(.identity | startswith("kw-plugins:") | not)] | length')
  if [ "$bad_identities" -eq 0 ]; then
    _pass "kw-plugins identities all start with kw-plugins:"
  else
    _fail "kw-plugins identity prefix" "kw-plugins:*" "some identities have wrong prefix"
  fi

  # Verify identity format: source:plugin/skill (contains exactly one colon and one slash after it)
  local malformed
  malformed=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s) | .identity | select(test("^kw-plugins:[^/]+/[^/]+$") | not)] | length')
  if [ "$malformed" -eq 0 ]; then
    _pass "kw-plugins identities all match source:plugin/skill format"
  else
    local sample
    sample=$(echo "$json" | jq -r --arg s "$source" \
      '[.[] | select(.source == $s) | .identity | select(test("^kw-plugins:[^/]+/[^/]+$") | not)] | .[0]')
    _fail "kw-plugins identity format" "kw-plugins:<plugin>/<skill>" "$sample"
  fi

  _cleanup_source "$source"
}

# ---------------------------------------------------------------------------
# 13. anthropics/knowledge-work-plugins/tree/main/productivity — SinglePlugin
# ---------------------------------------------------------------------------
test_13_knowledge_work_productivity() {
  skip_if_no_network && return

  local source="kw-prod"
  local json
  json=$(_add_and_list "https://github.com/anthropics/knowledge-work-plugins/tree/main/productivity" "$source")

  if echo "$json" | grep -q "^EXIT:"; then
    _fail "add failed for knowledge-work-plugins/tree/main/productivity" "exit 0" "$json"
    return
  fi

  local plugin_count skill_count
  skill_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | length')
  plugin_count=$(echo "$json" | jq --arg s "$source" '[.[] | select(.source == $s)] | [.[].plugin] | unique | length')

  if [ "$plugin_count" -eq 1 ]; then
    _pass "kw-productivity: SinglePlugin (1 plugin)"
  else
    _fail "kw-productivity plugin count" "1" "$plugin_count"
  fi

  if [ "$skill_count" -ge 3 ]; then
    _pass "kw-productivity has $skill_count skills"
  else
    _fail "kw-productivity skill count" ">=3" "$skill_count"
  fi

  # Identity: SinglePlugin plugin name from plugin.json = "productivity"
  # All identities should match kw-prod:productivity/<skill>
  local bad_identities
  bad_identities=$(echo "$json" | jq -r --arg s "$source" \
    '[.[] | select(.source == $s) | select(.identity | startswith("kw-prod:productivity/") | not)] | length')
  if [ "$bad_identities" -eq 0 ]; then
    _pass "kw-prod identities all match kw-prod:productivity/*"
  else
    local sample
    sample=$(echo "$json" | jq -r --arg s "$source" '[.[] | select(.source == $s)] | .[0].identity')
    _fail "kw-prod identity format" "kw-prod:productivity/*" "$sample"
  fi

  _cleanup_source "$source"
}
