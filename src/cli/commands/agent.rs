use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    add_detected_agents, copy_dir_all, detect_agents, generate_marketplace,
};
use crate::cli::AgentCommand;

/// Known built-in agent types
const KNOWN_AGENTS: &[&str] = &["claude", "codex", "cursor", "gemini", "vscode"];

pub(crate) fn run(command: AgentCommand, flags: &Flags) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;

    match command {
        AgentCommand::Add {
            agent,
            path,
            name,
            scope,
            sync,
        } => {
            // Validate agent type against built-in + custom adapters
            if !KNOWN_AGENTS.contains(&agent.as_str())
                && !config.adapter.contains_key(&agent)
            {
                let available: Vec<String> = KNOWN_AGENTS
                    .iter()
                    .map(|s| s.to_string())
                    .chain(config.adapter.keys().cloned())
                    .collect();
                anyhow::bail!(
                    "unknown agent type '{}'. Available: {}",
                    agent,
                    available.join(", ")
                );
            }

            let agent_name = name.unwrap_or_else(|| format!("{}-{}", agent, scope));

            // Check for duplicate name
            if config.agent.iter().any(|t| t.name == agent_name) {
                anyhow::bail!("agent '{}' already exists", agent_name);
            }

            // Resolve path: default based on agent + scope
            let agent_path = if let Some(p) = path {
                std::path::PathBuf::from(p)
            } else {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("~"))
                    .join(format!(".{}", agent))
            };

            // Repo scope defaults to explicit sync
            let actual_sync = if scope == "repo" && sync == "auto" {
                "explicit".to_string()
            } else {
                sync
            };

            if !flags.dry_run {
                config.agent.push(crate::config::AgentConfig {
                    name: agent_name.clone(),
                    agent_type: agent.clone(),
                    path: agent_path,
                    scope,
                    sync: actual_sync,
                });
                crate::config::save(&config, config_path_str)?;
            }

            if !flags.quiet {
                if flags.dry_run {
                    println!("  (dry run) would add agent '{}'", agent_name);
                } else {
                    println!("Added agent '{}'", agent_name);
                }
            }
            Ok(())
        }
        AgentCommand::Remove { name, force } => {
            if !config.agent.iter().any(|t| t.name == name) {
                anyhow::bail!("agent '{}' not found", name);
            }

            let execute = force && !flags.dry_run;
            if execute {
                config.agent.retain(|t| t.name != name);
                crate::config::save(&config, config_path_str)?;
            }

            if !flags.quiet {
                if execute {
                    println!("Removed agent '{}' (installed skills preserved)", name);
                } else {
                    println!("Would remove agent '{}'", name);
                    println!("Use --force to remove");
                }
            }
            Ok(())
        }
        AgentCommand::List => {
            if flags.json {
                let entries: Vec<serde_json::Value> = config
                    .agent
                    .iter()
                    .map(|t| {
                        let adapter = crate::agent::resolve_adapter(t, &config.adapter).ok();
                        let installed_count = adapter
                            .as_ref()
                            .and_then(|a| a.installed_skills(&t.path).ok())
                            .map(|v| v.len())
                            .unwrap_or(0);
                        serde_json::json!({
                            "name": t.name,
                            "agent": t.agent_type,
                            "path": t.path,
                            "scope": t.scope,
                            "sync": t.sync,
                            "installed": installed_count,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&entries)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            if config.agent.is_empty() {
                out.info("No agents configured. Use `equip agent add` to add one.");
                return Ok(());
            }

            for ac in &config.agent {
                let adapter = crate::agent::resolve_adapter(ac, &config.adapter).ok();
                let installed = adapter
                    .as_ref()
                    .and_then(|a| a.installed_skills(&ac.path).ok())
                    .unwrap_or_default();

                println!(
                    "{} {} {}",
                    ac.name.bold(),
                    format!("({})", ac.agent_type).cyan(),
                    format!("— {}", ac.path.display()).dimmed(),
                );
                println!(
                    "  {} {} {}",
                    "scope:".dimmed(),
                    ac.scope,
                    format!("  sync: {}  installed: {}", ac.sync, installed.len()).dimmed(),
                );
            }
            Ok(())
        }
        AgentCommand::Show { name } => {
            let agent_cfg = config
                .agent
                .iter()
                .find(|t| t.name == name)
                .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", name))?;

            if flags.json {
                let json = serde_json::json!({
                    "name": agent_cfg.name,
                    "type": agent_cfg.agent_type,
                    "path": agent_cfg.path,
                    "scope": agent_cfg.scope,
                    "sync": agent_cfg.sync,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            out.status("Name", &agent_cfg.name);
            out.status("Type", &agent_cfg.agent_type);
            out.status("Path", &agent_cfg.path.display().to_string());
            out.status("Scope", &agent_cfg.scope);
            out.status("Sync", &agent_cfg.sync);

            // List installed skills if the directory exists
            let skills_dir = agent_cfg.path.join("skills");
            if skills_dir.is_dir() {
                let mut installed = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                            && entry.path().join("SKILL.md").exists()
                        {
                            installed.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                if !installed.is_empty() {
                    installed.sort();
                    out.status("Installed", &installed.len().to_string());
                    out.info("");
                    let tree: Vec<(usize, String)> =
                        installed.into_iter().map(|s| (0, s)).collect();
                    out.tree(&tree);
                }
            }

            Ok(())
        }
        AgentCommand::Detect { force } => {
            let candidates = detect_agents();

            let mut found: Vec<(String, std::path::PathBuf, bool)> = Vec::new();
            for (agent, path) in &candidates {
                let already = config.agent.iter().any(|t| t.path == *path);
                found.push((agent.clone(), path.clone(), already));
            }

            if flags.json {
                let entries: Vec<serde_json::Value> = found
                    .iter()
                    .map(|(agent, path, registered)| {
                        serde_json::json!({
                            "agent": agent,
                            "path": path,
                            "registered": registered,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&entries)?);
                return Ok(());
            }

            if found.is_empty() {
                if !flags.quiet {
                    println!("No agent configurations found.");
                }
                return Ok(());
            }

            if force {
                let added = add_detected_agents(&mut config, flags.quiet);
                if added > 0 {
                    crate::config::save(&config, config_path_str)?;
                }
            } else {
                let home = std::env::var("HOME")
                    .map(std::path::PathBuf::from)
                    .or_else(|_| dirs::home_dir().ok_or(()))
                    .unwrap_or_else(|_| std::path::PathBuf::from("~"));
                let out =
                    crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
                let mut added = 0;
                for (agent, path, registered) in &found {
                    if *registered {
                        out.info(&format!(
                            "{} at {} (already registered)",
                            agent,
                            path.display()
                        ));
                        continue;
                    }
                    let agent_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(agent)
                        .trim_start_matches('.')
                        .to_string();
                    eprint!(
                        "Add {} at {} as agent '{}'? [y/N] ",
                        agent,
                        path.display(),
                        agent_name
                    );
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap_or(0);
                    if input.trim().eq_ignore_ascii_case("y") {
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
                        out.success(&format!("Added agent '{}'", agent_name));
                        added += 1;
                    }
                }
                if added > 0 {
                    crate::config::save(&config, config_path_str)?;
                }
            }

            Ok(())
        }
        AgentCommand::Collect {
            agent,
            skill,
            adopt,
            force,
        } => {
            let data_dir = crate::config::data_dir();
            let mut registry = crate::registry::load_registry(&data_dir)?;
            let renames = crate::registry::reconcile_with_config(
                &mut registry,
                &config.source,
                &data_dir,
            )?;
            if !renames.is_empty() {
                crate::registry::save_registry(&registry, &data_dir)?;
                if !flags.quiet {
                    for r in &renames {
                        eprintln!("source renamed: {}", r);
                    }
                }
            }
            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);

            let ac = config
                .agent
                .iter()
                .find(|a| a.name == agent)
                .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", agent))?;
            let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;
            let installed_on_agent = adapter.installed_skills(&ac.path)?;

            if let Some(ref skill_name) = skill {
                let agent_skill_dir = ac.path.join("skills").join(skill_name);
                if !agent_skill_dir.exists() {
                    anyhow::bail!("skill '{}' not found on agent '{}'", skill_name, agent);
                }

                if adopt {
                    let provenance = registry
                        .installed
                        .get(&agent)
                        .and_then(|m| m.get(skill_name));
                    let plugin_name = provenance
                        .map(|info| info.plugin.clone())
                        .unwrap_or_else(|| "local".to_string());
                    let source_name = provenance
                        .map(|info| info.source.clone())
                        .unwrap_or_else(|| "local".to_string());

                    let dest_plugin = crate::config::plugins_dir().join(&plugin_name);
                    let dest_skill = dest_plugin.join("skills").join(skill_name);
                    std::fs::create_dir_all(&dest_skill)?;
                    copy_dir_all(&agent_skill_dir, &dest_skill)?;

                    let plugin_json_dir = dest_plugin.join(".claude-plugin");
                    let plugin_json = plugin_json_dir.join("plugin.json");
                    if !plugin_json.exists() {
                        std::fs::create_dir_all(&plugin_json_dir)?;
                        let json = serde_json::json!({"name": plugin_name});
                        std::fs::write(&plugin_json, serde_json::to_string_pretty(&json)?)?;
                    }

                    generate_marketplace(&data_dir)?;
                    let identity =
                        crate::output::format_identity(&source_name, &plugin_name, skill_name);
                    out.success(&format!("Adopted {}", identity));
                } else {
                    let provenance = registry
                        .installed
                        .get(&agent)
                        .and_then(|m| m.get(skill_name));

                    if let Some(info) = provenance {
                        let dest = data_dir.join(&info.origin);
                        std::fs::create_dir_all(&dest)?;
                        copy_dir_all(&agent_skill_dir, &dest)?;
                        let identity =
                            crate::output::format_identity(&info.source, &info.plugin, &info.skill);
                        out.success(&format!("Collected {} → {}", identity, info.origin));
                    } else {
                        out.warn(&format!(
                            "'{}' has no provenance. Use --adopt to claim it.",
                            skill_name
                        ));
                    }
                }
            } else {
                let agent_installs = registry.installed.get(&agent).cloned().unwrap_or_default();

                let mut tracked = Vec::new();
                let mut untracked = Vec::new();

                for skill_name in &installed_on_agent {
                    if let Some(info) = agent_installs.get(skill_name) {
                        tracked.push((skill_name.clone(), info.clone()));
                    } else {
                        untracked.push(skill_name.clone());
                    }
                }

                if !tracked.is_empty() {
                    out.info("Tracked:");
                    for (_name, info) in &tracked {
                        let identity =
                            crate::output::format_identity(&info.source, &info.plugin, &info.skill);
                        out.info(&format!("  {} ← {}", identity, info.origin));
                    }
                }

                if !untracked.is_empty() {
                    out.info("Untracked:");
                    for name in &untracked {
                        out.info(&format!("  {}", name));
                    }

                    let should_adopt = if force {
                        true
                    } else if !untracked.is_empty() {
                        eprint!(
                            "Adopt {} untracked skill(s) into local/? [y/N] ",
                            untracked.len()
                        );
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap_or(0);
                        input.trim().eq_ignore_ascii_case("y")
                    } else {
                        false
                    };

                    if should_adopt {
                        let local_plugin = crate::config::plugins_dir().join("local");
                        for name in &untracked {
                            let agent_skill_dir = ac.path.join("skills").join(name);
                            let dest = local_plugin.join("skills").join(name);
                            std::fs::create_dir_all(&dest)?;
                            copy_dir_all(&agent_skill_dir, &dest)?;
                            let identity = crate::output::format_identity("local", "local", name);
                            out.success(&format!("Adopted {}", identity));
                        }

                        let plugin_json_dir = local_plugin.join(".claude-plugin");
                        let plugin_json = plugin_json_dir.join("plugin.json");
                        if !plugin_json.exists() {
                            std::fs::create_dir_all(&plugin_json_dir)?;
                            let json = serde_json::json!({"name": "local"});
                            std::fs::write(&plugin_json, serde_json::to_string_pretty(&json)?)?;
                        }

                        generate_marketplace(&data_dir)?;
                    }
                }

                if tracked.is_empty() && untracked.is_empty() {
                    out.info("No skills found on agent.");
                }
            }

            crate::registry::save_registry(&registry, &data_dir)?;
            Ok(())
        }
    }
}
