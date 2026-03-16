use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    add_detected_agents, copy_dir_all, detect_agents, generate_marketplace, print_apply_summary,
    prompt_conflict, record_provenance, resolve_agents, ConflictAction,
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
                out.info("No agents configured. Use `loadout agent add` to add one.");
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
        AgentCommand::Equip {
            patterns,
            agent,
            all,
            kit,
            save,
            force,
            interactive,
        } => {
            // Parse @agent and +kit prefixes from positional patterns
            let mut agent = agent;
            let mut kit = kit;
            let mut skill_patterns: Vec<String> = Vec::new();
            for pat in &patterns {
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
                    skill_patterns.push(pat.clone());
                }
            }

            if skill_patterns.is_empty() && kit.is_none() {
                eprintln!("error: equip requires skill patterns or a kit (+name / --kit)");
                std::process::exit(2);
            }

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

            let agents = resolve_agents(&config, &agent, all)?;

            // Collect skills to equip with provenance: (source, plugin, skill)
            let mut skills_to_apply: Vec<(&str, &str, &crate::registry::RegisteredSkill)> =
                Vec::new();

            // From positional patterns
            for pattern in &skill_patterns {
                if crate::registry::is_glob(pattern) {
                    let matches = registry.match_skills(pattern);
                    if matches.is_empty() {
                        anyhow::bail!("no skills matched pattern '{}'", pattern);
                    }
                    for (src, plugin, s) in matches {
                        skills_to_apply.push((src, &plugin.name, s));
                    }
                } else {
                    match registry.find_skill(pattern) {
                        Ok((src, plug, s)) => {
                            skills_to_apply.push((src, plug, s));
                        }
                        Err(_) => {
                            let matches = registry.match_skills(pattern);
                            if matches.is_empty() {
                                anyhow::bail!("no skills matched '{}'", pattern);
                            }
                            for (src, plugin, s) in matches {
                                skills_to_apply.push((src, &plugin.name, s));
                            }
                        }
                    }
                }
            }

            // From --kit / +kit — resolve skills, track whether kit exists
            let kit_exists = if let Some(ref kit_name) = kit {
                match config.kit.get(kit_name) {
                    Some(kit_cfg) => {
                        for skill_id in &kit_cfg.skills {
                            let (src, plug, s) = registry.find_skill(skill_id)?;
                            skills_to_apply.push((src, plug, s));
                        }
                        true
                    }
                    None if save && !skill_patterns.is_empty() => false,
                    None => {
                        anyhow::bail!("kit '{}' not found", kit_name);
                    }
                }
            } else {
                false
            };

            // Interactive confirmation: show skills, prompt for kit creation, then proceed
            if !force && !flags.dry_run && !skills_to_apply.is_empty() && crate::prompt::is_interactive() {
                eprintln!("Skills to equip:");
                for (src, plug, s) in &skills_to_apply {
                    eprintln!("  {}", crate::output::format_identity(src, plug, &s.name));
                }
                eprintln!("Agents:");
                for ac in &agents {
                    eprintln!("  {}", ac.name.bold());
                }

                // Prompt to create missing kit before proceeding
                if save && !kit_exists {
                    if let Some(ref kit_name) = kit {
                        eprint!(
                            "Create kit '{}' ({} skill{})? [y/N] ",
                            kit_name,
                            skills_to_apply.len(),
                            if skills_to_apply.len() == 1 { "" } else { "s" },
                        );
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap_or(0);
                        if input.trim().eq_ignore_ascii_case("y") {
                            let mut skill_ids: Vec<String> = Vec::new();
                            for (src, plug, s) in &skills_to_apply {
                                let fq = crate::output::plain_identity(src, plug, &s.name);
                                if !skill_ids.contains(&fq) {
                                    skill_ids.push(fq);
                                }
                            }
                            let mut save_config = crate::config::load(flags.config_path())?;
                            save_config.kit.insert(
                                kit_name.clone(),
                                crate::config::KitConfig { skills: skill_ids },
                            );
                            crate::config::save(&save_config, flags.config_path())?;
                            if !flags.quiet {
                                println!("Created kit '{}'", kit_name);
                            }
                        } else {
                            eprintln!("Aborted.");
                            return Ok(());
                        }
                    }
                }

                eprint!("Proceed? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap_or(0);
                if !input.trim().eq_ignore_ascii_case("y") {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            } else if save && !kit_exists {
                // Non-interactive / --force: create kit silently
                if let Some(ref kit_name) = kit {
                    let mut skill_ids: Vec<String> = Vec::new();
                    for (src, plug, s) in &skills_to_apply {
                        let fq = crate::output::plain_identity(src, plug, &s.name);
                        if !skill_ids.contains(&fq) {
                            skill_ids.push(fq);
                        }
                    }
                    let mut save_config = crate::config::load(flags.config_path())?;
                    save_config.kit.insert(
                        kit_name.clone(),
                        crate::config::KitConfig { skills: skill_ids },
                    );
                    crate::config::save(&save_config, flags.config_path())?;
                    if !flags.quiet {
                        println!("Created kit '{}'", kit_name);
                    }
                }
            }

            // Apply to each agent with conflict detection
            let mut new_count: usize = 0;
            let mut updated_count: usize = 0;
            let mut unchanged_count: usize = 0;
            let mut conflict_skipped: usize = 0;
            let mut force_remaining = force;
            let mut reg = registry.clone();

            for ac in &agents {
                let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;

                // Detect conflicts when not interactive and not forced
                if !force && !interactive && !flags.dry_run {
                    let mut conflicts = Vec::new();
                    for (_, _, s) in &skills_to_apply {
                        let status = adapter.compare_skill(s, &ac.path)?;
                        if status == crate::agent::SkillStatus::Changed {
                            conflicts.push(s.name.clone());
                        }
                    }
                    if !conflicts.is_empty() {
                        eprintln!(
                            "error: {} skill(s) have changed at agent '{}':",
                            conflicts.len(),
                            ac.name
                        );
                        for name in &conflicts {
                            eprintln!("  - {}", name);
                        }
                        eprintln!();
                        eprintln!("Use --force to overwrite, or -i for interactive resolution.");
                        std::process::exit(1);
                    }
                }

                for (src_name, plug_name, s) in &skills_to_apply {
                    let status = adapter.compare_skill(s, &ac.path)?;

                    if flags.dry_run {
                        if !flags.quiet {
                            let label = match status {
                                crate::agent::SkillStatus::New => "new",
                                crate::agent::SkillStatus::Unchanged => "unchanged",
                                crate::agent::SkillStatus::Changed => "changed",
                            };
                            println!(
                                "  (dry run) {} → {} [{}]",
                                crate::output::format_identity(src_name, plug_name, &s.name),
                                ac.name,
                                label
                            );
                        }
                        continue;
                    }

                    match status {
                        crate::agent::SkillStatus::Unchanged => {
                            unchanged_count += 1;
                            continue;
                        }
                        crate::agent::SkillStatus::New => {
                            adapter.install_skill(s, &ac.path)?;
                            record_provenance(&mut reg, &data_dir, ac, src_name, plug_name, s);
                            new_count += 1;
                        }
                        crate::agent::SkillStatus::Changed => {
                            if force_remaining {
                                adapter.install_skill(s, &ac.path)?;
                                record_provenance(&mut reg, &data_dir, ac, src_name, plug_name, s);
                                updated_count += 1;
                            } else if interactive {
                                let action = prompt_conflict(s, &adapter, &ac.path)?;
                                match action {
                                    ConflictAction::Skip => {
                                        conflict_skipped += 1;
                                    }
                                    ConflictAction::Overwrite => {
                                        adapter.install_skill(s, &ac.path)?;
                                        record_provenance(
                                            &mut reg, &data_dir, ac, src_name, plug_name, s,
                                        );
                                        updated_count += 1;
                                    }
                                    ConflictAction::ForceAll => {
                                        adapter.install_skill(s, &ac.path)?;
                                        record_provenance(
                                            &mut reg, &data_dir, ac, src_name, plug_name, s,
                                        );
                                        updated_count += 1;
                                        force_remaining = true;
                                    }
                                    ConflictAction::Quit => {
                                        crate::registry::save_registry(&reg, &data_dir)?;
                                        print_apply_summary(
                                            new_count,
                                            updated_count,
                                            unchanged_count,
                                            conflict_skipped,
                                            flags.quiet,
                                        );
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !flags.dry_run {
                crate::registry::save_registry(&reg, &data_dir)?;
            }

            // --save: update existing kit with resolved skill set
            if save && kit_exists {
                let kit_name = kit.as_ref().unwrap();
                let mut config = crate::config::load(flags.config_path())?;
                let mut skill_ids: Vec<String> = Vec::new();
                for (src, plug, s) in &skills_to_apply {
                    let fq = crate::output::plain_identity(src, plug, &s.name);
                    if !skill_ids.contains(&fq) {
                        skill_ids.push(fq);
                    }
                }

                let should_save = if force || !crate::prompt::is_interactive() {
                    true
                } else {
                    eprint!(
                        "Update kit '{}' ({} skill{})? [y/N] ",
                        kit_name,
                        skill_ids.len(),
                        if skill_ids.len() == 1 { "" } else { "s" },
                    );
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap_or(0);
                    input.trim().eq_ignore_ascii_case("y")
                };

                if should_save {
                    config.kit.insert(
                        kit_name.clone(),
                        crate::config::KitConfig { skills: skill_ids },
                    );
                    crate::config::save(&config, flags.config_path())?;
                    if !flags.quiet {
                        println!("Updated kit '{}'", kit_name);
                    }
                }
            } else if save && kit.is_none() {
                anyhow::bail!("--save requires --kit (or +name) to specify the kit name");
            }

            if !flags.quiet && !flags.dry_run {
                print_apply_summary(
                    new_count,
                    updated_count,
                    unchanged_count,
                    conflict_skipped,
                    flags.quiet,
                );
            }
            Ok(())
        }
        AgentCommand::Unequip {
            patterns,
            agent,
            all,
            kit,
            force,
        } => {
            // Parse @agent and +kit prefixes from positional patterns
            let mut agent = agent;
            let mut kit = kit;
            let mut skill_patterns: Vec<String> = Vec::new();
            for pat in &patterns {
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
                    skill_patterns.push(pat.clone());
                }
            }

            if skill_patterns.is_empty() && kit.is_none() {
                eprintln!("error: unequip requires skill patterns or a kit (+name / --kit)");
                std::process::exit(2);
            }

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

            let agents = resolve_agents(&config, &agent, all)?;

            // Collect skill names to remove
            let mut skill_names: Vec<String> = Vec::new();

            for pattern in &skill_patterns {
                if crate::registry::is_glob(pattern) {
                    let matches = registry.match_skills(pattern);
                    if matches.is_empty() {
                        anyhow::bail!("no skills matched pattern '{}'", pattern);
                    }
                    for (_, _, s) in matches {
                        if !skill_names.contains(&s.name) {
                            skill_names.push(s.name.clone());
                        }
                    }
                } else {
                    match registry.find_skill(pattern) {
                        Ok((_, _, s)) => {
                            if !skill_names.contains(&s.name) {
                                skill_names.push(s.name.clone());
                            }
                        }
                        Err(_) => {
                            let matches = registry.match_skills(pattern);
                            if matches.is_empty() {
                                anyhow::bail!("no skills matched '{}'", pattern);
                            }
                            for (_, _, s) in matches {
                                if !skill_names.contains(&s.name) {
                                    skill_names.push(s.name.clone());
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref kit_name) = kit {
                match config.kit.get(kit_name) {
                    Some(kit_cfg) => {
                        for skill_id in &kit_cfg.skills {
                            let (_, _, s) = registry.find_skill(skill_id)?;
                            if !skill_names.contains(&s.name) {
                                skill_names.push(s.name.clone());
                            }
                        }
                    }
                    None => {
                        anyhow::bail!("kit '{}' not found", kit_name);
                    }
                }
            }

            let execute = force && !flags.dry_run;
            let mut total_removed = 0usize;
            let mut _total_skipped = 0usize;

            for ac in &agents {
                let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;
                let installed = adapter.installed_skills(&ac.path).unwrap_or_default();

                for name in &skill_names {
                    if installed.contains(name) {
                        let identity = registry
                            .installed
                            .get(&ac.name)
                            .and_then(|m| m.get(name))
                            .map(|info| {
                                crate::output::format_identity(&info.source, &info.plugin, &info.skill)
                            })
                            .unwrap_or_else(|| name.clone());

                        if execute {
                            adapter.uninstall_skill(name, &ac.path)?;
                            if let Some(agent_map) = registry.installed.get_mut(&ac.name) {
                                agent_map.remove(name);
                            }
                            out.success(&format!(
                                "Removed {} from {}",
                                identity,
                                ac.name.bold()
                            ));
                            total_removed += 1;
                        } else {
                            out.info(&format!("  {} from {}", identity, ac.name.bold()));
                            total_removed += 1;
                        }
                    } else {
                        _total_skipped += 1;
                    }
                }
            }

            if !execute && total_removed > 0 {
                out.info("");
                out.warn("Preview only. Use --force to execute.");
            }

            if execute {
                crate::registry::save_registry(&registry, &data_dir)?;
                if !flags.quiet {
                    out.info(&format!(
                        "Removed {} skill(s) from {} agent(s)",
                        total_removed,
                        agents.len()
                    ));
                }
            } else if total_removed == 0 && !flags.quiet {
                out.info("No matching skills found on agent(s).");
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
                            "Adopt {} untracked skill(s) into plugins/local? [y/N] ",
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
