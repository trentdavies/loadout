use colored::Colorize;
use std::collections::BTreeSet;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    add_detected_agents, detect_agents, load_effective_registry, resolve_skills_for_bundle,
};
use crate::cli::AgentCommand;

/// Known built-in agent types
const KNOWN_AGENTS: &[&str] = &["claude", "codex", "cursor", "gemini", "vscode"];

#[derive(Debug, Clone)]
struct InstalledSkillView {
    installed_name: String,
    display: String,
    identity: Option<String>,
    tracked: bool,
}

fn installed_skill_views(
    agent_cfg: &crate::config::AgentConfig,
    config: &crate::config::Config,
    registry: &crate::registry::Registry,
    colorize: bool,
) -> Vec<InstalledSkillView> {
    let adapter = match crate::agent::resolve_adapter(agent_cfg, &config.adapter) {
        Ok(adapter) => adapter,
        Err(_) => return Vec::new(),
    };

    let installed_names = adapter
        .installed_skills(&agent_cfg.path)
        .unwrap_or_default();
    let installed_index = registry.installed.get(&agent_cfg.name);

    installed_names
        .into_iter()
        .map(|installed_name| {
            if let Some(installed) = installed_index.and_then(|skills| skills.get(&installed_name))
            {
                let identity = crate::output::plain_identity(
                    &installed.source,
                    &installed.plugin,
                    &installed.skill,
                );
                let mut display = if colorize {
                    crate::output::format_identity(
                        &installed.source,
                        &installed.plugin,
                        &installed.skill,
                    )
                } else {
                    identity.clone()
                };

                if installed.skill != installed_name {
                    display.push_str(&format!(
                        " {}",
                        format!("(installed as {})", installed_name).dimmed()
                    ));
                }

                InstalledSkillView {
                    installed_name,
                    display,
                    identity: Some(identity),
                    tracked: true,
                }
            } else {
                let display = if colorize {
                    format!("{} {}", installed_name.bold(), "(untracked)".yellow())
                } else {
                    format!("{} (untracked)", installed_name)
                };

                InstalledSkillView {
                    installed_name,
                    display,
                    identity: None,
                    tracked: false,
                }
            }
        })
        .collect()
}

fn installed_kit_names(
    config: &crate::config::Config,
    registry: &crate::registry::Registry,
    installed: &[InstalledSkillView],
) -> Vec<String> {
    let installed_identities: BTreeSet<String> = installed
        .iter()
        .filter_map(|skill| skill.identity.clone())
        .collect();

    let mut kits = Vec::new();
    for (kit_name, kit) in &config.kit {
        if kit.skills.is_empty() {
            continue;
        }

        let mut fully_installed = true;
        for spec in &kit.skills {
            let resolved = match resolve_skills_for_bundle(spec, registry) {
                Ok(resolved) if !resolved.is_empty() => resolved,
                _ => {
                    fully_installed = false;
                    break;
                }
            };

            if !resolved
                .iter()
                .any(|(_, identity)| installed_identities.contains(identity))
            {
                fully_installed = false;
                break;
            }
        }

        if fully_installed {
            kits.push(kit_name.clone());
        }
    }

    kits
}

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
            if !KNOWN_AGENTS.contains(&agent.as_str()) && !config.adapter.contains_key(&agent) {
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
        AgentCommand::List {
            show_skills,
            show_kits,
        } => {
            let data_dir = crate::config::data_dir();
            let registry = load_effective_registry(&config, &data_dir, flags.quiet)?;
            if flags.json {
                let entries: Vec<serde_json::Value> = config
                    .agent
                    .iter()
                    .map(|t| {
                        let installed = installed_skill_views(t, &config, &registry, false);
                        let kits = if show_kits {
                            installed_kit_names(&config, &registry, &installed)
                        } else {
                            Vec::new()
                        };

                        let mut entry = serde_json::Map::new();
                        entry.insert("name".to_string(), serde_json::json!(t.name));
                        entry.insert("agent".to_string(), serde_json::json!(t.agent_type));
                        entry.insert("path".to_string(), serde_json::json!(t.path));
                        entry.insert("scope".to_string(), serde_json::json!(t.scope));
                        entry.insert("sync".to_string(), serde_json::json!(t.sync));
                        entry.insert("installed".to_string(), serde_json::json!(installed.len()));

                        if show_skills {
                            entry.insert(
                                "skills".to_string(),
                                serde_json::Value::Array(
                                    installed
                                        .iter()
                                        .map(|skill| {
                                            serde_json::json!({
                                                "name": skill.installed_name,
                                                "tracked": skill.tracked,
                                                "identity": skill.identity,
                                            })
                                        })
                                        .collect(),
                                ),
                            );
                        }

                        if show_kits {
                            entry.insert("kits".to_string(), serde_json::json!(kits));
                        }

                        serde_json::Value::Object(entry)
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
                let installed = installed_skill_views(ac, &config, &registry, true);
                let kits = if show_kits {
                    installed_kit_names(&config, &registry, &installed)
                } else {
                    Vec::new()
                };

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

                if show_skills {
                    println!("  {}", "skills:".dimmed());
                    if installed.is_empty() {
                        println!("    {}", "none".dimmed());
                    } else {
                        let tree: Vec<(usize, String)> = installed
                            .iter()
                            .map(|skill| (1, skill.display.clone()))
                            .collect();
                        out.tree(&tree);
                    }
                }

                if show_kits {
                    println!("  {}", "kits:".dimmed());
                    if kits.is_empty() {
                        println!("    {}", "none".dimmed());
                    } else {
                        let tree: Vec<(usize, String)> = kits
                            .into_iter()
                            .map(|kit| (1, format!("{}", format!("+{}", kit).magenta())))
                            .collect();
                        out.tree(&tree);
                    }
                }
            }
            Ok(())
        }
        AgentCommand::Show { name } => {
            let agent_cfg = config
                .agent
                .iter()
                .find(|t| t.name == name)
                .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", name))?;
            let data_dir = crate::config::data_dir();
            let registry = load_effective_registry(&config, &data_dir, flags.quiet)?;
            let installed = installed_skill_views(agent_cfg, &config, &registry, !flags.json);

            if flags.json {
                let json = serde_json::json!({
                    "name": agent_cfg.name,
                    "type": agent_cfg.agent_type,
                    "path": agent_cfg.path,
                    "scope": agent_cfg.scope,
                    "sync": agent_cfg.sync,
                    "installed": installed.iter().map(|skill| {
                        serde_json::json!({
                            "name": skill.installed_name,
                            "tracked": skill.tracked,
                            "identity": skill.identity,
                        })
                    }).collect::<Vec<_>>(),
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

            if !installed.is_empty() {
                out.status("Installed", &installed.len().to_string());
                out.info("");
                let tree: Vec<(usize, String)> = installed
                    .into_iter()
                    .map(|skill| (0, skill.display))
                    .collect();
                out.tree(&tree);
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
                let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
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
            patterns,
            kit,
            link,
            adopt_local,
            force,
            interactive,
        } => crate::cli::commands::collect::run(
            agent,
            patterns,
            kit,
            link,
            adopt_local,
            force,
            interactive,
            flags,
        ),
    }
}
