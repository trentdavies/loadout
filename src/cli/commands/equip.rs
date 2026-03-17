use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    print_apply_summary, prompt_conflict, record_provenance, resolve_agents, ConflictAction,
};

pub(crate) fn run(
    patterns: Vec<String>,
    agent: Option<Vec<String>>,
    all: bool,
    kit: Option<String>,
    save: bool,
    force: bool,
    interactive: bool,
    remove: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    if remove {
        if save {
            anyhow::bail!("--remove cannot be combined with --save");
        }
        if interactive {
            anyhow::bail!("--remove cannot be combined with --interactive");
        }
        run_unequip(patterns, agent, all, kit, force, flags)
    } else {
        run_equip(patterns, agent, all, kit, save, force, interactive, flags)
    }
}

fn run_equip(
    patterns: Vec<String>,
    agent: Option<Vec<String>>,
    all: bool,
    kit: Option<String>,
    save: bool,
    force: bool,
    interactive: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let config = crate::config::load(config_path_str)?;

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
            None if !skill_patterns.is_empty() => {
                anyhow::bail!("kit '{}' not found; add -s to create '{}'", kit_name, kit_name);
            }
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

        eprint!(
            "Install {} skill{} to {} agent{}? [y/N] ",
            skills_to_apply.len(),
            if skills_to_apply.len() == 1 { "" } else { "s" },
            agents.len(),
            if agents.len() == 1 { "" } else { "s" },
        );
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

fn run_unequip(
    patterns: Vec<String>,
    agent: Option<Vec<String>>,
    all: bool,
    kit: Option<String>,
    force: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let config = crate::config::load(config_path_str)?;

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
