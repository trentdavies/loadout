use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    apply_skill_to_agent, load_context, parse_apply_selection, persist_kit_selection,
    print_apply_summary, resolve_agents, resolve_skill_patterns, unique_skill_names,
    ApplySkillOutcome, PersistKitMode, PersistKitResult, ResolvedSkill,
};

pub(crate) struct EquipArgs {
    pub patterns: Vec<String>,
    pub agent: Option<Vec<String>>,
    pub all: bool,
    pub kit: Option<String>,
    pub save: bool,
    pub force: bool,
    pub interactive: bool,
    pub remove: bool,
}

pub(crate) fn run(args: EquipArgs, flags: &Flags) -> anyhow::Result<()> {
    if args.remove {
        if args.save {
            anyhow::bail!("--remove cannot be combined with --save");
        }
        if args.interactive {
            anyhow::bail!("--remove cannot be combined with --interactive");
        }
        run_unequip(
            args.patterns,
            args.agent,
            args.all,
            args.kit,
            args.force,
            flags,
        )
    } else {
        run_equip(args, flags)
    }
}

fn run_equip(args: EquipArgs, flags: &Flags) -> anyhow::Result<()> {
    let EquipArgs {
        patterns,
        agent,
        all,
        kit,
        save,
        force,
        interactive,
        ..
    } = args;
    let ctx = load_context(flags)?;
    let config = ctx.config;

    let selection = parse_apply_selection(patterns, agent, kit)?;
    let agent = selection.agent;
    let kit = selection.kit;
    let skill_patterns = selection.skill_patterns;

    if skill_patterns.is_empty() && kit.is_none() {
        eprintln!("error: equip requires skill patterns or a kit (+name / --kit)");
        std::process::exit(2);
    }

    let data_dir = ctx.data_dir;
    let registry = ctx.registry;

    let agents = resolve_agents(&config, &agent, all)?;

    // Collect skills to equip with provenance: (source, plugin, skill)
    let mut skills_to_apply: Vec<ResolvedSkill<'_>> = Vec::new();

    // From positional patterns
    skills_to_apply.extend(resolve_skill_patterns(&skill_patterns, &registry, false)?);

    // From --kit / +kit — resolve skills, track whether kit exists
    let kit_exists = if let Some(ref kit_name) = kit {
        match config.kit.get(kit_name) {
            Some(kit_cfg) => {
                skills_to_apply.extend(resolve_skill_patterns(&kit_cfg.skills, &registry, false)?);
                true
            }
            None if save && !skill_patterns.is_empty() => false,
            None if !skill_patterns.is_empty() => {
                anyhow::bail!(
                    "kit '{}' not found; add -s to create '{}'",
                    kit_name,
                    kit_name
                );
            }
            None => {
                anyhow::bail!("kit '{}' not found", kit_name);
            }
        }
    } else {
        false
    };
    let skill_ids = save.then(|| crate::cli::helpers::fully_qualified_skill_ids(&skills_to_apply));

    // Interactive confirmation: show skills, prompt for kit creation, then proceed
    if !force && !flags.dry_run && !skills_to_apply.is_empty() && crate::prompt::is_interactive() {
        eprintln!("Skills to equip:");
        for (src, plugin, s) in &skills_to_apply {
            eprintln!(
                "  {}",
                crate::output::format_identity(src, &plugin.name, &s.name)
            );
        }
        eprintln!("Agents:");
        for ac in &agents {
            eprintln!("  {}", ac.id.bold());
        }

        // Prompt to create missing kit before proceeding
        if save && !kit_exists {
            if let Some(ref kit_name) = kit {
                if matches!(
                    persist_kit_selection(
                        flags,
                        kit_name,
                        skill_ids.as_deref().unwrap_or(&[]),
                        PersistKitMode::Create,
                        false,
                    )?,
                    PersistKitResult::Aborted
                ) {
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
            persist_kit_selection(
                flags,
                kit_name,
                skill_ids.as_deref().unwrap_or(&[]),
                PersistKitMode::Create,
                true,
            )?;
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
                    ac.id
                );
                for name in &conflicts {
                    eprintln!("  - {}", name);
                }
                eprintln!();
                eprintln!("Use --force to overwrite, or -i for interactive resolution.");
                std::process::exit(1);
            }
        }

        for (src_name, plugin, s) in &skills_to_apply {
            if flags.dry_run {
                let status = adapter.compare_skill(s, &ac.path)?;
                if !flags.quiet {
                    let label = match status {
                        crate::agent::SkillStatus::New => "new",
                        crate::agent::SkillStatus::Unchanged => "unchanged",
                        crate::agent::SkillStatus::Changed => "changed",
                    };
                    println!(
                        "  (dry run) {} → {} [{}]",
                        crate::output::format_identity(src_name, &plugin.name, &s.name),
                        ac.id,
                        label
                    );
                }
                continue;
            }

            match apply_skill_to_agent(
                &adapter,
                &mut reg,
                &data_dir,
                ac,
                (*src_name, *plugin, *s),
                interactive,
                &mut force_remaining,
            )? {
                ApplySkillOutcome::Unchanged => {
                    unchanged_count += 1;
                }
                ApplySkillOutcome::New => {
                    new_count += 1;
                }
                ApplySkillOutcome::Updated => {
                    updated_count += 1;
                }
                ApplySkillOutcome::ConflictSkipped => {
                    conflict_skipped += 1;
                }
                ApplySkillOutcome::Quit => {
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

    if !flags.dry_run {
        crate::registry::save_registry(&reg, &data_dir)?;
    }

    // --save: update existing kit with resolved skill set
    if save && kit_exists {
        let kit_name = kit.as_ref().unwrap();
        persist_kit_selection(
            flags,
            kit_name,
            skill_ids.as_deref().unwrap_or(&[]),
            PersistKitMode::Update,
            force || !crate::prompt::is_interactive(),
        )?;
    } else if save && kit.is_none() {
        anyhow::bail!("--save requires --kit (or +name) to specify the kit name");
    }

    // Sync equipped list and save config after all kit operations
    if !flags.dry_run {
        // Reload config to pick up any kit changes from persist_kit_selection
        let mut final_config = crate::config::load(flags.config_path())?;
        crate::cli::helpers::sync_equipped_from_installed(&mut final_config, &reg);
        crate::config::save(&final_config, flags.config_path())?;
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
    let ctx = load_context(flags)?;
    let mut config = ctx.config;

    let selection = parse_apply_selection(patterns, agent, kit)?;
    let agent = selection.agent;
    let kit = selection.kit;
    let skill_patterns = selection.skill_patterns;

    if skill_patterns.is_empty() && kit.is_none() {
        eprintln!("error: unequip requires skill patterns or a kit (+name / --kit)");
        std::process::exit(2);
    }

    let data_dir = ctx.data_dir;
    let mut registry = ctx.registry;
    let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);

    let agents = resolve_agents(&config, &agent, all)?;

    let mut resolved_skills = resolve_skill_patterns(&skill_patterns, &registry, true)?;

    if let Some(ref kit_name) = kit {
        match config.kit.get(kit_name) {
            Some(kit_cfg) => {
                resolved_skills.extend(resolve_skill_patterns(&kit_cfg.skills, &registry, true)?);
            }
            None => {
                anyhow::bail!("kit '{}' not found", kit_name);
            }
        }
    }
    let skill_names = unique_skill_names(&resolved_skills);

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
                    .get(&ac.id)
                    .and_then(|m| m.get(name))
                    .map(|info| {
                        crate::output::format_identity(&info.source, &info.plugin, &info.skill)
                    })
                    .unwrap_or_else(|| name.clone());

                if execute {
                    adapter.uninstall_skill(name, &ac.path)?;
                    if let Some(agent_map) = registry.installed.get_mut(&ac.id) {
                        agent_map.remove(name);
                    }
                    out.success(&format!("Removed {} from {}", identity, ac.id.bold()));
                    total_removed += 1;
                } else {
                    out.info(&format!("  {} from {}", identity, ac.id.bold()));
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

    let agent_count = agents.len();
    drop(agents); // release borrow on config

    if execute {
        crate::registry::save_registry(&registry, &data_dir)?;
        crate::cli::helpers::sync_equipped_from_installed(&mut config, &registry);
        crate::config::save(&config, flags.config_path())?;
        if !flags.quiet {
            out.info(&format!(
                "Removed {} skill(s) from {} agent(s)",
                total_removed, agent_count
            ));
        }
    } else if total_removed == 0 && !flags.quiet {
        out.info("No matching skills found on agent(s).");
    }

    Ok(())
}
