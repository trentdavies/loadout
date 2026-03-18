use colored::Colorize;

pub(crate) const AGENT_PREFIXES: &[(&str, &str)] = &[
    ("claude", ".claude"),
    ("codex", ".codex"),
    ("cursor", ".cursor"),
];

pub(crate) type ResolvedSkill<'a> = (
    &'a str,
    &'a crate::registry::RegisteredPlugin,
    &'a crate::registry::RegisteredSkill,
);

pub(crate) struct CommandContext {
    pub config: crate::config::Config,
    pub registry: crate::registry::Registry,
    pub data_dir: std::path::PathBuf,
    pub source_labels: std::collections::BTreeMap<String, String>,
}

pub(crate) struct ApplySelection {
    pub agent: Option<Vec<String>>,
    pub kit: Option<String>,
    pub skill_patterns: Vec<String>,
}

pub(crate) enum PersistKitMode {
    Create,
    Update,
}

pub(crate) enum PersistKitResult {
    Saved,
    Skipped,
    Aborted,
}

/// Extract the domain from a URL. Returns empty string for local paths.
pub(crate) fn extract_domain(url: &str) -> String {
    let (host, path) = if let Some(rest) = url.strip_prefix("git@") {
        let mut parts = rest.splitn(2, ':');
        let h = parts.next().unwrap_or("");
        let p = parts.next().unwrap_or("");
        (h.to_string(), p.to_string())
    } else if let Some(after_scheme) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git://"))
        .or_else(|| url.strip_prefix("ssh://"))
    {
        let mut parts = after_scheme.splitn(2, '/');
        let h = parts.next().unwrap_or("");
        let h = h.split('@').next_back().unwrap_or(h);
        let p = parts.next().unwrap_or("");
        (h.to_string(), p.to_string())
    } else {
        return String::new();
    };

    if host == "github.com" {
        let slug = path.trim_end_matches(".git");
        if !slug.is_empty() {
            return format!("Github: {slug}");
        }
    }

    host
}

/// Build a set of source names stored in the external cache.
pub(crate) fn external_source_set(
    config: &crate::config::Config,
) -> std::collections::HashSet<String> {
    config
        .source
        .iter()
        .filter(|s| s.residence == crate::config::SourceResidence::External)
        .map(|s| s.name.clone())
        .collect()
}

/// Format a breakdown like "3 external, 2 local" or just "3 external" / "3 local".
pub(crate) fn source_breakdown(external: usize, local: usize) -> String {
    match (external, local) {
        (0, l) => format!("{} local", l),
        (e, 0) => format!("{} external", e),
        (e, l) => format!("{} external, {} local", e, l),
    }
}

/// Resolve a skill identifier (exact, glob, or freeform) to a list of (source_name, fully_qualified_id).
pub(crate) fn resolve_skills_for_bundle(
    skill_id: &str,
    registry: &crate::registry::Registry,
) -> anyhow::Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    for (src, plugin, skill) in resolve_skill_query(skill_id, registry)? {
        let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
        results.push((src.to_string(), fq));
    }
    Ok(results)
}

pub(crate) fn load_context(flags: &crate::cli::flags::Flags) -> anyhow::Result<CommandContext> {
    let config = crate::config::load(flags.config_path())?;
    let data_dir = crate::config::data_dir();
    let registry = load_effective_registry(&config, &data_dir, flags.quiet)?;
    let source_labels = build_source_labels(&registry, &data_dir);

    Ok(CommandContext {
        config,
        registry,
        data_dir,
        source_labels,
    })
}

pub(crate) fn load_effective_registry(
    config: &crate::config::Config,
    data_dir: &std::path::Path,
    quiet: bool,
) -> anyhow::Result<crate::registry::Registry> {
    let mut registry = crate::registry::load_registry(data_dir)?;
    let renames = crate::registry::reconcile_with_config(&mut registry, &config.source, data_dir)?;
    if !renames.is_empty() {
        crate::registry::save_registry(&registry, data_dir)?;
        if !quiet {
            for rename in &renames {
                eprintln!("source reconciled: {}", rename);
            }
        }
    }

    merge_local_source(&mut registry, data_dir)?;
    Ok(registry)
}

fn merge_local_source(
    registry: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
) -> anyhow::Result<()> {
    registry.sources.retain(|source| source.name != "local");

    let parsed = match crate::source::ParsedSource::parse(data_dir) {
        Ok(parsed) => parsed,
        Err(_) => return Ok(()),
    };
    if parsed.kind != crate::source::SourceKind::Marketplace {
        return Ok(());
    }

    let mut local_source = crate::source::normalize::normalize(
        &parsed.with_source_name("local").with_url(""),
    )?;
    local_source.name = "local".to_string();
    local_source.url.clear();
    local_source.residence = crate::config::SourceResidence::Local;
    registry.sources.push(local_source);
    Ok(())
}

fn build_source_labels(
    registry: &crate::registry::Registry,
    _data_dir: &std::path::Path,
) -> std::collections::BTreeMap<String, String> {
    let mut labels = std::collections::BTreeMap::new();
    for source in &registry.sources {
        labels.insert(
            source.name.clone(),
            source
                .display_name
                .clone()
                .unwrap_or_else(|| source.name.clone()),
        );
    }

    labels
}

pub(crate) fn parse_apply_selection(
    patterns: Vec<String>,
    mut agent: Option<Vec<String>>,
    mut kit: Option<String>,
) -> anyhow::Result<ApplySelection> {
    let mut skill_patterns = Vec::new();

    for pat in patterns {
        if let Some(name) = pat.strip_prefix('@') {
            let agents = agent.get_or_insert_with(Vec::new);
            if !agents.contains(&name.to_string()) {
                agents.push(name.to_string());
            }
        } else if let Some(name) = pat.strip_prefix('+') {
            if kit.is_some() {
                anyhow::bail!("multiple kits specified (--kit and +{})", name);
            }
            kit = Some(name.to_string());
        } else {
            skill_patterns.push(pat);
        }
    }

    Ok(ApplySelection {
        agent,
        kit,
        skill_patterns,
    })
}

pub(crate) fn resolve_skill_query<'a>(
    skill_id: &str,
    registry: &'a crate::registry::Registry,
) -> anyhow::Result<Vec<ResolvedSkill<'a>>> {
    if crate::registry::is_glob(skill_id) {
        let matches = registry.match_skills(skill_id);
        if matches.is_empty() {
            anyhow::bail!("no skills matched pattern '{}'", skill_id);
        }
        return Ok(matches);
    }

    match registry.find_skill_entry(skill_id) {
        Ok(skill) => Ok(vec![skill]),
        Err(_) => {
            let matches = registry.match_skills(skill_id);
            if matches.is_empty() {
                anyhow::bail!("no skills matched '{}'", skill_id);
            }
            Ok(matches)
        }
    }
}

pub(crate) fn resolve_skill_patterns<'a>(
    patterns: &[String],
    registry: &'a crate::registry::Registry,
    dedupe: bool,
) -> anyhow::Result<Vec<ResolvedSkill<'a>>> {
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for pattern in patterns {
        for triple in resolve_skill_query(pattern, registry)? {
            let id = crate::output::plain_identity(triple.0, &triple.1.name, &triple.2.name);
            if !dedupe || seen.insert(id) {
                results.push(triple);
            }
        }
    }

    Ok(results)
}

pub(crate) fn fully_qualified_skill_ids(skills: &[ResolvedSkill<'_>]) -> Vec<String> {
    let mut ids = Vec::new();
    for (src, plugin, skill) in skills {
        let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
        if !ids.contains(&fq) {
            ids.push(fq);
        }
    }
    ids
}

pub(crate) fn unique_skill_names(skills: &[ResolvedSkill<'_>]) -> Vec<String> {
    let mut names = Vec::new();
    for (_, _, skill) in skills {
        if !names.contains(&skill.name) {
            names.push(skill.name.clone());
        }
    }
    names
}

pub(crate) fn persist_kit_selection(
    flags: &crate::cli::flags::Flags,
    kit_name: &str,
    skill_ids: &[String],
    mode: PersistKitMode,
    auto_confirm: bool,
) -> anyhow::Result<PersistKitResult> {
    let should_save = if auto_confirm || !crate::prompt::is_interactive() {
        true
    } else {
        eprint!(
            "{} kit '{}' ({} skill{})? [y/N] ",
            match mode {
                PersistKitMode::Create => "Create",
                PersistKitMode::Update => "Update",
            },
            kit_name,
            skill_ids.len(),
            if skill_ids.len() == 1 { "" } else { "s" },
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or(0);
        input.trim().eq_ignore_ascii_case("y")
    };

    if !should_save {
        return Ok(match mode {
            PersistKitMode::Create => PersistKitResult::Aborted,
            PersistKitMode::Update => PersistKitResult::Skipped,
        });
    }

    let mut config = crate::config::load(flags.config_path())?;
    config.kit.insert(
        kit_name.to_string(),
        crate::config::KitConfig {
            skills: skill_ids.to_vec(),
        },
    );
    crate::config::save(&config, flags.config_path())?;

    if !flags.quiet {
        println!(
            "{} kit '{}'",
            match mode {
                PersistKitMode::Create => "Created",
                PersistKitMode::Update => "Updated",
            },
            kit_name
        );
    }

    Ok(PersistKitResult::Saved)
}

/// Scan home and cwd for agent installation directories.
/// Returns (agent_type, path) for each found candidate.
pub fn detect_agents() -> Vec<(String, std::path::PathBuf)> {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| dirs::home_dir().ok_or(()))
        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

    let cwd = std::env::current_dir().unwrap_or_default();
    let dirs_to_scan: Vec<&std::path::Path> = if cwd == home {
        vec![&home]
    } else {
        vec![&home, &cwd]
    };

    let mut candidates: Vec<(String, std::path::PathBuf)> = Vec::new();
    for scan_dir in &dirs_to_scan {
        if let Ok(entries) = std::fs::read_dir(scan_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                for (agent, prefix) in AGENT_PREFIXES {
                    if name_str == *prefix || name_str.starts_with(&format!("{}-", prefix)) {
                        let path = entry.path();
                        if !candidates.iter().any(|(_, p)| *p == path) {
                            candidates.push((agent.to_string(), path));
                        }
                    }
                }
            }
        }
    }
    candidates.sort_by(|a, b| a.1.cmp(&b.1));
    candidates
}

/// Add all detected agents to config (auto-add, no per-agent prompt).
/// Returns count of agents added.
pub fn add_detected_agents(config: &mut crate::config::Config, quiet: bool) -> usize {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| dirs::home_dir().ok_or(()))
        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

    let candidates = detect_agents();
    let mut added = 0;
    for (agent, path) in &candidates {
        if config.agent.iter().any(|t| t.path == *path) {
            continue;
        }
        let agent_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(agent)
            .trim_start_matches('.')
            .to_string();
        let scope = if path.starts_with(&home) {
            "machine"
        } else {
            "repo"
        };
        let sync = if scope == "repo" { "explicit" } else { "auto" };
        config.agent.push(crate::config::AgentConfig {
            name: agent_name.clone(),
            agent_type: agent.to_string(),
            path: path.clone(),
            scope: scope.to_string(),
            sync: sync.to_string(),
        });
        if !quiet {
            let display_path = if let Some(home_str) = home.to_str() {
                path.to_string_lossy().replacen(home_str, "~", 1)
            } else {
                path.to_string_lossy().to_string()
            };
            let skills_desc = match agent.as_str() {
                "cursor" => format!("(skills in {}/)", display_path),
                _ => format!("(skills in {}/skills/)", display_path),
            };
            println!(
                "  {} {}  {}  {}",
                "✓".green(),
                agent_name.bold(),
                display_path.dimmed(),
                skills_desc.dimmed(),
            );
        }
        added += 1;
    }
    added
}

pub(crate) fn resolve_agents<'a>(
    config: &'a crate::config::Config,
    agent_names: &Option<Vec<String>>,
    all_agents: bool,
) -> anyhow::Result<Vec<&'a crate::config::AgentConfig>> {
    if all_agents {
        if config.agent.is_empty() {
            anyhow::bail!("no agents configured. Use `equip agent add` first.");
        }
        return Ok(config.agent.iter().collect());
    }

    if let Some(names) = agent_names {
        let mut agents = Vec::new();
        for name in names {
            let ac = config
                .agent
                .iter()
                .find(|ac| ac.name == *name)
                .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", name))?;
            agents.push(ac);
        }
        if agents.is_empty() {
            anyhow::bail!("no agents specified");
        }
        return Ok(agents);
    }

    let auto: Vec<_> = config.agent.iter().filter(|t| t.sync == "auto").collect();
    if auto.is_empty() {
        anyhow::bail!("no agents configured. Use `equip agent add` first.");
    }
    Ok(auto)
}

/// Record provenance for an applied skill.
pub(crate) fn record_provenance(
    reg: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    ac: &crate::config::AgentConfig,
    src_name: &str,
    plug_name: &str,
    s: &crate::registry::RegisteredSkill,
) {
    let origin = s
        .path
        .strip_prefix(data_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| s.path.display().to_string());
    let agent_map = reg.installed.entry(ac.name.clone()).or_default();
    agent_map.insert(
        s.name.clone(),
        crate::registry::InstalledSkill {
            source: src_name.to_string(),
            plugin: plug_name.to_string(),
            skill: s.name.clone(),
            origin,
        },
    );
}

pub(crate) enum ApplySkillOutcome {
    New,
    Updated,
    Unchanged,
    ConflictSkipped,
    Quit,
}

pub(crate) fn apply_skill_to_agent(
    adapter: &crate::agent::Adapter,
    reg: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    ac: &crate::config::AgentConfig,
    resolved_skill: ResolvedSkill<'_>,
    interactive: bool,
    force_all: &mut bool,
) -> anyhow::Result<ApplySkillOutcome> {
    let (src_name, plugin, skill) = resolved_skill;

    match adapter.compare_skill(skill, &ac.path)? {
        crate::agent::SkillStatus::Unchanged => Ok(ApplySkillOutcome::Unchanged),
        crate::agent::SkillStatus::New => {
            adapter.install_skill(skill, &ac.path)?;
            record_provenance(reg, data_dir, ac, src_name, &plugin.name, skill);
            Ok(ApplySkillOutcome::New)
        }
        crate::agent::SkillStatus::Changed if *force_all => {
            adapter.install_skill(skill, &ac.path)?;
            record_provenance(reg, data_dir, ac, src_name, &plugin.name, skill);
            Ok(ApplySkillOutcome::Updated)
        }
        crate::agent::SkillStatus::Changed if interactive => {
            match prompt_conflict(skill, adapter, &ac.path)? {
                ConflictAction::Skip => Ok(ApplySkillOutcome::ConflictSkipped),
                ConflictAction::Overwrite => {
                    adapter.install_skill(skill, &ac.path)?;
                    record_provenance(reg, data_dir, ac, src_name, &plugin.name, skill);
                    Ok(ApplySkillOutcome::Updated)
                }
                ConflictAction::ForceAll => {
                    adapter.install_skill(skill, &ac.path)?;
                    record_provenance(reg, data_dir, ac, src_name, &plugin.name, skill);
                    *force_all = true;
                    Ok(ApplySkillOutcome::Updated)
                }
                ConflictAction::Quit => Ok(ApplySkillOutcome::Quit),
            }
        }
        crate::agent::SkillStatus::Changed => anyhow::bail!(
            "skill '{}' has changed at agent '{}'; use --force or -i to continue",
            skill.name,
            ac.name
        ),
    }
}

/// Action chosen by user in interactive conflict resolution.
pub(crate) enum ConflictAction {
    Skip,
    Overwrite,
    ForceAll,
    Quit,
}

/// Prompt the user to resolve a conflict for a changed skill.
pub(crate) fn prompt_conflict(
    skill: &crate::registry::RegisteredSkill,
    adapter: &crate::agent::Adapter,
    agent_path: &std::path::Path,
) -> anyhow::Result<ConflictAction> {
    eprintln!();
    eprintln!("  {} — CHANGED", skill.name);
    eprintln!();
    eprint!("    [s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit: ");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        match choice.as_str() {
            "s" => return Ok(ConflictAction::Skip),
            "o" => return Ok(ConflictAction::Overwrite),
            "f" => return Ok(ConflictAction::ForceAll),
            "q" => return Ok(ConflictAction::Quit),
            "d" => {
                show_skill_diff(skill, adapter, agent_path)?;
                eprintln!();
                eprint!("    [s]kip  [o]verwrite  [q]uit: ");
                let mut input2 = String::new();
                std::io::stdin().read_line(&mut input2)?;
                let choice2 = input2.trim().to_lowercase();
                match choice2.as_str() {
                    "s" => return Ok(ConflictAction::Skip),
                    "o" => return Ok(ConflictAction::Overwrite),
                    "q" => return Ok(ConflictAction::Quit),
                    _ => {
                        eprint!("    Invalid choice. [s]kip  [o]verwrite  [q]uit: ");
                        continue;
                    }
                }
            }
            _ => {
                eprint!("    Invalid choice. [s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit: ");
                continue;
            }
        }
    }
}

/// Display a unified diff of all files in a skill directory.
fn show_skill_diff(
    skill: &crate::registry::RegisteredSkill,
    adapter: &crate::agent::Adapter,
    agent_path: &std::path::Path,
) -> anyhow::Result<()> {
    let pairs = adapter.skill_file_pairs(skill, agent_path)?;

    for (label, src_path, dst_path) in &pairs {
        let src_content = if src_path.exists() {
            std::fs::read_to_string(src_path).unwrap_or_default()
        } else {
            String::new()
        };
        let dst_content = if dst_path.exists() {
            std::fs::read_to_string(dst_path).unwrap_or_default()
        } else {
            String::new()
        };

        if src_content == dst_content {
            continue;
        }

        eprintln!();
        eprintln!("    === {} ===", label);

        let diff = similar::TextDiff::from_lines(&dst_content, &src_content);
        for hunk in diff
            .unified_diff()
            .header("installed", "source")
            .iter_hunks()
        {
            eprint!("    {}", hunk);
        }
    }

    Ok(())
}

/// Print the apply summary line.
pub(crate) fn print_apply_summary(
    new_count: usize,
    updated_count: usize,
    unchanged_count: usize,
    conflict_skipped: usize,
    quiet: bool,
) {
    if quiet {
        return;
    }
    let applied = new_count + updated_count;
    let mut msg = format!(
        "Applied {} skill(s) ({} new, {} updated), skipped {} unchanged.",
        applied, new_count, updated_count, unchanged_count
    );
    if conflict_skipped > 0 {
        msg.push_str(&format!(" {} conflict skipped.", conflict_skipped));
    }
    println!("{}", msg);
}

pub(crate) fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Generate .claude-plugin/marketplace.json from plugin directories in the data dir root.
pub(crate) fn generate_marketplace(data_dir: &std::path::Path) -> anyhow::Result<()> {
    crate::marketplace::generate_local_manifest(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill(dir: &std::path::Path, name: &str) {
        let skill_dir = dir.join("skills").join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {}\ndescription: desc\n---\nbody", name),
        )
        .unwrap();
    }

    fn make_plugin(dir: &std::path::Path, name: &str) {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            format!(r#"{{"name":"{}"}}"#, name),
        )
        .unwrap();
        make_skill(&plugin_dir, "skill-a");
    }

    #[test]
    fn load_effective_registry_includes_local_source() {
        let tmp = TempDir::new().unwrap();
        make_plugin(tmp.path(), "tools");
        fs::create_dir_all(tmp.path().join(".claude-plugin")).unwrap();
        fs::write(
            tmp.path().join(".claude-plugin/marketplace.json"),
            r#"{"name":"Team Skills","plugins":[{"name":"tools","source":"./tools"}]}"#,
        )
        .unwrap();

        let registry = load_effective_registry(&crate::config::Config::default(), tmp.path(), true)
            .unwrap();
        let local = registry
            .sources
            .iter()
            .find(|source| source.name == "local")
            .unwrap();

        assert_eq!(local.residence, crate::config::SourceResidence::Local);
        assert_eq!(local.plugins.len(), 1);
        assert_eq!(local.plugins[0].name, "tools");
        assert_eq!(local.plugins[0].skills[0].name, "skill-a");
    }

    #[test]
    fn build_source_labels_uses_local_marketplace_name() {
        let tmp = TempDir::new().unwrap();
        make_plugin(tmp.path(), "tools");
        fs::create_dir_all(tmp.path().join(".claude-plugin")).unwrap();
        fs::write(
            tmp.path().join(".claude-plugin/marketplace.json"),
            r#"{"name":"Pretty Local","plugins":[{"name":"tools","source":"./tools"}]}"#,
        )
        .unwrap();

        let registry = load_effective_registry(&crate::config::Config::default(), tmp.path(), true)
            .unwrap();
        let labels = build_source_labels(&registry, tmp.path());

        assert_eq!(labels.get("local").map(String::as_str), Some("Pretty Local"));
    }

    #[test]
    fn generate_marketplace_preserves_existing_name() {
        let tmp = TempDir::new().unwrap();
        make_plugin(tmp.path(), "tools");
        fs::create_dir_all(tmp.path().join(".claude-plugin")).unwrap();
        fs::write(
            tmp.path().join(".claude-plugin/marketplace.json"),
            r#"{"name":"Shared Skills","plugins":[]}"#,
        )
        .unwrap();

        generate_marketplace(tmp.path()).unwrap();

        let manifest =
            crate::source::manifest::load_marketplace(&tmp.path().join(".claude-plugin/marketplace.json"))
                .unwrap();
        assert_eq!(manifest.name, "Shared Skills");
        assert_eq!(manifest.plugins.len(), 1);
        assert_eq!(manifest.plugins[0].name, "tools");
    }
}
