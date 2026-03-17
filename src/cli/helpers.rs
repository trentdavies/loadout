use colored::Colorize;

pub(crate) const AGENT_PREFIXES: &[(&str, &str)] = &[
    ("claude", ".claude"),
    ("codex", ".codex"),
    ("cursor", ".cursor"),
];

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

/// Build a set of source names that are external (git).
pub(crate) fn external_source_set(config: &crate::config::Config) -> std::collections::HashSet<String> {
    config
        .source
        .iter()
        .filter(|s| s.source_type == "git")
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
    if crate::registry::is_glob(skill_id) {
        let matches = registry.match_skills(skill_id);
        if matches.is_empty() {
            anyhow::bail!("no skills matched pattern '{}'", skill_id);
        }
        for (src, plugin, skill) in &matches {
            let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
            results.push((src.to_string(), fq));
        }
    } else {
        match registry.find_skill(skill_id) {
            Ok((src, plug, sk)) => {
                let fq = crate::output::plain_identity(src, plug, &sk.name);
                results.push((src.to_string(), fq));
            }
            Err(_) => {
                let matches = registry.match_skills(skill_id);
                if matches.is_empty() {
                    anyhow::bail!("no skills matched '{}'", skill_id);
                }
                for (src, plugin, skill) in &matches {
                    let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
                    results.push((src.to_string(), fq));
                }
            }
        }
    }
    Ok(results)
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
    /// Directories in the data dir that are infrastructure, not plugins.
    const SKIP_DIRS: &[&str] = &["external"];

    let mut plugins = Vec::new();

    if data_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(data_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.') || SKIP_DIRS.contains(&dir_name.as_str()) {
                continue;
            }

            // Only include directories that look like plugins (have skills/ or .claude-plugin/)
            let has_plugin_marker = entry.path().join(".claude-plugin").is_dir();
            let has_skills = entry.path().join("skills").is_dir();
            if !has_plugin_marker && !has_skills {
                continue;
            }

            let plugin_json = entry.path().join(".claude-plugin/plugin.json");
            let (name, description) = if plugin_json.exists() {
                if let Ok(manifest) = crate::source::manifest::load_plugin_manifest(&plugin_json) {
                    (manifest.name, manifest.description)
                } else {
                    (dir_name.clone(), None)
                }
            } else {
                (dir_name.clone(), None)
            };

            let mut plugin_entry = serde_json::json!({
                "name": name,
                "source": format!("./{}", dir_name),
            });
            if let Some(desc) = description {
                plugin_entry["description"] = serde_json::Value::String(desc);
            }
            plugins.push(plugin_entry);
        }
    }

    let marketplace = serde_json::json!({
        "name": "equip-marketplace",
        "plugins": plugins,
    });

    let cp_dir = data_dir.join(".claude-plugin");
    std::fs::create_dir_all(&cp_dir)?;
    std::fs::write(
        cp_dir.join("marketplace.json"),
        serde_json::to_string_pretty(&marketplace)?,
    )?;

    Ok(())
}
