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
    /// Source names that are virtual (generated from agent-installed untracked skills).
    pub virtual_sources: std::collections::HashSet<String>,
}

/// Status of a skill in `equip list`, modeled after git status indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillListStatus {
    /// Exists on agent but not in any source.
    Untracked,
    /// Tracked, installed on an agent, but the agent copy differs from source.
    Modified,
    /// Tracked and up to date (or not installed on any agent).
    Clean,
}

impl SkillListStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Untracked => "untracked",
            Self::Modified => "modified",
            Self::Clean => "clean",
        }
    }
}

/// Compute the list status for a skill.
pub(crate) fn skill_list_status(
    source_name: &str,
    plugin: &crate::registry::RegisteredPlugin,
    skill: &crate::registry::RegisteredSkill,
    ctx: &CommandContext,
) -> SkillListStatus {
    if ctx.virtual_sources.contains(source_name) {
        return SkillListStatus::Untracked;
    }

    // Check if any agent has this skill installed
    for ac in &ctx.config.agent {
        let has_install = ctx
            .registry
            .installed
            .get(&ac.name)
            .and_then(|agent_skills| {
                agent_skills.values().find(|installed| {
                    installed.source == source_name
                        && installed.plugin == plugin.name
                        && installed.skill == skill.name
                })
            })
            .is_some();

        if !has_install {
            continue;
        }

        let adapter = match crate::agent::resolve_adapter(ac, &ctx.config.adapter) {
            Ok(a) => a,
            Err(_) => continue,
        };

        if let Ok(crate::agent::SkillStatus::Changed) = adapter.compare_skill(skill, &ac.path) {
            return SkillListStatus::Modified;
        }
    }

    SkillListStatus::Clean
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
    let mut registry = load_effective_registry(&config, &data_dir, flags.quiet)?;
    let virtual_sources = merge_agent_sources(&mut registry, &config);
    let source_labels = build_source_labels(&registry, &data_dir);

    Ok(CommandContext {
        config,
        registry,
        data_dir,
        source_labels,
        virtual_sources,
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

    let mut local_source =
        crate::source::normalize::normalize(&parsed.with_source_name("local").with_url(""))?;
    local_source.name = "local".to_string();
    local_source.url.clear();
    local_source.residence = crate::config::SourceResidence::Local;
    registry.sources.push(local_source);
    Ok(())
}

/// Scan each configured agent for untracked skills and inject a virtual
/// `RegisteredSource` per agent so they appear in `equip list`.
/// Returns the set of source names that are virtual (agent-derived).
fn merge_agent_sources(
    registry: &mut crate::registry::Registry,
    config: &crate::config::Config,
) -> std::collections::HashSet<String> {
    let mut virtual_names = std::collections::HashSet::new();

    for ac in &config.agent {
        let adapter = match crate::agent::resolve_adapter(ac, &config.adapter) {
            Ok(a) => a,
            Err(_) => continue,
        };
        let installed_names = adapter.installed_skills(&ac.path).unwrap_or_default();
        if installed_names.is_empty() {
            continue;
        }

        let installed_index = registry.installed.get(&ac.name);

        let mut untracked_skills = Vec::new();
        for name in &installed_names {
            let tracked = installed_index
                .and_then(|skills| skills.get(name))
                .is_some();
            if tracked {
                continue;
            }
            // Read description from SKILL.md if available
            let skill_dir = adapter.skill_dest(&ac.path, name);
            let skill_md = skill_dir.join("SKILL.md");
            let description = if skill_md.exists() {
                crate::source::detect::parse_skill_description(&skill_md)
            } else {
                None
            };
            untracked_skills.push(crate::registry::RegisteredSkill {
                name: name.clone(),
                description,
                author: None,
                version: None,
                path: skill_dir,
            });
        }

        if untracked_skills.is_empty() {
            continue;
        }

        let source_name = ac.name.clone();
        virtual_names.insert(source_name.clone());

        // Remove any existing virtual source with the same name (from a prior merge)
        registry
            .sources
            .retain(|s| s.name != source_name || !s.url.is_empty());

        registry
            .sources
            .push(crate::registry::RegisteredSource {
                name: source_name.clone(),
                display_name: None,
                url: String::new(),
                plugins: vec![crate::registry::RegisteredPlugin {
                    name: source_name.clone(),
                    version: None,
                    description: None,
                    skills: untracked_skills,
                    path: ac.path.clone(),
                }],
                cache_path: ac.path.clone(),
                residence: crate::config::SourceResidence::External,
            });
    }

    virtual_names
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
    record_provenance_as(reg, data_dir, ac, &s.name, src_name, plug_name, s);
}

/// Record provenance for an installed skill name that maps to a canonical skill.
pub(crate) fn record_provenance_as(
    reg: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    ac: &crate::config::AgentConfig,
    installed_name: &str,
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
        installed_name.to_string(),
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
    let provenance_matches = reg
        .installed
        .get(&ac.name)
        .and_then(|agent_map| agent_map.get(&skill.name))
        .map(|installed| {
            installed.source == src_name
                && installed.plugin == plugin.name
                && installed.skill == skill.name
                && installed.origin
                    == skill
                        .path
                        .strip_prefix(data_dir)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| skill.path.display().to_string())
        })
        .unwrap_or(false);

    match adapter.compare_skill(skill, &ac.path)? {
        crate::agent::SkillStatus::Unchanged if *force_all && !provenance_matches => {
            adapter.install_skill(skill, &ac.path)?;
            record_provenance(reg, data_dir, ac, src_name, &plugin.name, skill);
            Ok(ApplySkillOutcome::Updated)
        }
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
    use std::collections::BTreeMap;
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

        let registry =
            load_effective_registry(&crate::config::Config::default(), tmp.path(), true).unwrap();
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

        let registry =
            load_effective_registry(&crate::config::Config::default(), tmp.path(), true).unwrap();
        let labels = build_source_labels(&registry, tmp.path());

        assert_eq!(
            labels.get("local").map(String::as_str),
            Some("Pretty Local")
        );
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

        let manifest = crate::source::manifest::load_marketplace(
            &tmp.path().join(".claude-plugin/marketplace.json"),
        )
        .unwrap();
        assert_eq!(manifest.name, "Shared Skills");
        assert_eq!(manifest.plugins.len(), 1);
        assert_eq!(manifest.plugins[0].name, "tools");
    }

    #[test]
    fn force_reapplies_when_identity_changes_but_contents_match() {
        let data_dir = TempDir::new().unwrap();
        let agent_dir = TempDir::new().unwrap();

        let skill_a_dir = data_dir
            .path()
            .join("external/source-a/plugin-a/skills/shared");
        fs::create_dir_all(&skill_a_dir).unwrap();
        fs::write(
            skill_a_dir.join("SKILL.md"),
            "---\nname: shared\ndescription: same\n---\nbody\n",
        )
        .unwrap();

        let skill_b_dir = data_dir
            .path()
            .join("external/source-b/plugin-b/skills/shared");
        fs::create_dir_all(&skill_b_dir).unwrap();
        fs::write(
            skill_b_dir.join("SKILL.md"),
            "---\nname: shared\ndescription: same\n---\nbody\n",
        )
        .unwrap();

        let agent = crate::config::AgentConfig {
            name: "claude".to_string(),
            agent_type: "claude".to_string(),
            path: agent_dir.path().to_path_buf(),
            scope: "machine".to_string(),
            sync: "auto".to_string(),
        };
        let adapter = crate::agent::resolve_adapter(&agent, &BTreeMap::new()).unwrap();

        let skill_a = crate::registry::RegisteredSkill {
            name: "shared".to_string(),
            description: Some("same".to_string()),
            author: None,
            version: None,
            path: skill_a_dir.clone(),
        };
        let skill_b = crate::registry::RegisteredSkill {
            name: "shared".to_string(),
            description: Some("same".to_string()),
            author: None,
            version: None,
            path: skill_b_dir.clone(),
        };
        let plugin_a = crate::registry::RegisteredPlugin {
            name: "plugin-a".to_string(),
            version: None,
            description: None,
            skills: vec![skill_a.clone()],
            path: data_dir.path().join("external/source-a/plugin-a"),
        };
        let plugin_b = crate::registry::RegisteredPlugin {
            name: "plugin-b".to_string(),
            version: None,
            description: None,
            skills: vec![skill_b.clone()],
            path: data_dir.path().join("external/source-b/plugin-b"),
        };

        let mut registry = crate::registry::Registry::default();
        adapter.install_skill(&skill_a, agent_dir.path()).unwrap();
        record_provenance(
            &mut registry,
            data_dir.path(),
            &agent,
            "source-a",
            &plugin_a.name,
            &skill_a,
        );

        let mut force_all = true;
        let outcome = apply_skill_to_agent(
            &adapter,
            &mut registry,
            data_dir.path(),
            &agent,
            ("source-b", &plugin_b, &skill_b),
            false,
            &mut force_all,
        )
        .unwrap();

        assert!(matches!(outcome, ApplySkillOutcome::Updated));

        let installed = &registry.installed["claude"]["shared"];
        assert_eq!(installed.source, "source-b");
        assert_eq!(installed.plugin, "plugin-b");
        assert_eq!(installed.skill, "shared");
        assert_eq!(installed.origin, "external/source-b/plugin-b/skills/shared");
    }

    #[test]
    fn merge_agent_sources_creates_virtual_source_for_untracked_skills() {
        let agent_dir = TempDir::new().unwrap();
        // Install two skills directly on the agent (no provenance)
        let skills_dir = agent_dir.path().join("skills");
        fs::create_dir_all(skills_dir.join("my-skill")).unwrap();
        fs::write(
            skills_dir.join("my-skill/SKILL.md"),
            "---\nname: my-skill\ndescription: A cool skill\n---\nbody",
        )
        .unwrap();
        fs::create_dir_all(skills_dir.join("other-skill")).unwrap();
        fs::write(
            skills_dir.join("other-skill/SKILL.md"),
            "---\nname: other-skill\ndescription: Another\n---\nbody",
        )
        .unwrap();

        let config = crate::config::Config {
            agent: vec![crate::config::AgentConfig {
                name: "test-agent".to_string(),
                agent_type: "claude".to_string(),
                path: agent_dir.path().to_path_buf(),
                scope: "machine".to_string(),
                sync: "auto".to_string(),
            }],
            ..Default::default()
        };

        let mut registry = crate::registry::Registry::default();
        let virtual_names = merge_agent_sources(&mut registry, &config);

        assert!(virtual_names.contains("test-agent"));
        assert_eq!(registry.sources.len(), 1);
        assert_eq!(registry.sources[0].name, "test-agent");
        assert_eq!(registry.sources[0].plugins.len(), 1);
        assert_eq!(registry.sources[0].plugins[0].skills.len(), 2);

        let skill_names: Vec<&str> = registry.sources[0].plugins[0]
            .skills
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(skill_names.contains(&"my-skill"));
        assert!(skill_names.contains(&"other-skill"));

        // Check description was parsed
        let my_skill = registry.sources[0].plugins[0]
            .skills
            .iter()
            .find(|s| s.name == "my-skill")
            .unwrap();
        assert_eq!(my_skill.description.as_deref(), Some("A cool skill"));
    }

    #[test]
    fn merge_agent_sources_excludes_tracked_skills() {
        let agent_dir = TempDir::new().unwrap();
        let skills_dir = agent_dir.path().join("skills");
        fs::create_dir_all(skills_dir.join("tracked-skill")).unwrap();
        fs::write(
            skills_dir.join("tracked-skill/SKILL.md"),
            "---\nname: tracked-skill\ndescription: tracked\n---\nbody",
        )
        .unwrap();
        fs::create_dir_all(skills_dir.join("untracked-skill")).unwrap();
        fs::write(
            skills_dir.join("untracked-skill/SKILL.md"),
            "---\nname: untracked-skill\ndescription: not tracked\n---\nbody",
        )
        .unwrap();

        let config = crate::config::Config {
            agent: vec![crate::config::AgentConfig {
                name: "my-agent".to_string(),
                agent_type: "claude".to_string(),
                path: agent_dir.path().to_path_buf(),
                scope: "machine".to_string(),
                sync: "auto".to_string(),
            }],
            ..Default::default()
        };

        let mut registry = crate::registry::Registry::default();
        // Mark tracked-skill as having provenance
        let mut agent_installed = BTreeMap::new();
        agent_installed.insert(
            "tracked-skill".to_string(),
            crate::registry::InstalledSkill {
                source: "some-source".to_string(),
                plugin: "some-plugin".to_string(),
                skill: "tracked-skill".to_string(),
                origin: "external/some-source/some-plugin/skills/tracked-skill".to_string(),
            },
        );
        registry
            .installed
            .insert("my-agent".to_string(), agent_installed);

        let virtual_names = merge_agent_sources(&mut registry, &config);

        assert!(virtual_names.contains("my-agent"));
        assert_eq!(registry.sources.len(), 1);
        // Only the untracked skill should be in the virtual source
        assert_eq!(registry.sources[0].plugins[0].skills.len(), 1);
        assert_eq!(
            registry.sources[0].plugins[0].skills[0].name,
            "untracked-skill"
        );
    }

    #[test]
    fn merge_agent_sources_skips_agents_with_no_untracked() {
        let agent_dir = TempDir::new().unwrap();
        let skills_dir = agent_dir.path().join("skills");
        fs::create_dir_all(skills_dir.join("tracked-only")).unwrap();
        fs::write(
            skills_dir.join("tracked-only/SKILL.md"),
            "---\nname: tracked-only\ndescription: all tracked\n---\nbody",
        )
        .unwrap();

        let config = crate::config::Config {
            agent: vec![crate::config::AgentConfig {
                name: "full-agent".to_string(),
                agent_type: "claude".to_string(),
                path: agent_dir.path().to_path_buf(),
                scope: "machine".to_string(),
                sync: "auto".to_string(),
            }],
            ..Default::default()
        };

        let mut registry = crate::registry::Registry::default();
        let mut agent_installed = BTreeMap::new();
        agent_installed.insert(
            "tracked-only".to_string(),
            crate::registry::InstalledSkill {
                source: "src".to_string(),
                plugin: "plug".to_string(),
                skill: "tracked-only".to_string(),
                origin: "external/src/plug/skills/tracked-only".to_string(),
            },
        );
        registry
            .installed
            .insert("full-agent".to_string(), agent_installed);

        let virtual_names = merge_agent_sources(&mut registry, &config);

        assert!(virtual_names.is_empty());
        assert!(registry.sources.is_empty());
    }
}
