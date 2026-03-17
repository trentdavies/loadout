use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{external_source_set, resolve_skills_for_bundle, source_breakdown};
use crate::cli::KitCommand;

pub(crate) fn run(command: KitCommand, flags: &Flags) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    match command {
        KitCommand::Create { name, skills } => {
            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            if config.kit.contains_key(&name) {
                anyhow::bail!("kit '{}' already exists", name);
            }
            config
                .kit
                .insert(name.clone(), crate::config::KitConfig::default());

            let mut added = 0usize;
            let mut external = 0usize;
            let mut local = 0usize;
            if !skills.is_empty() {
                let registry = crate::registry::load_registry(&data_dir)?;
                let ext_sources = external_source_set(&config);
                let kit = config.kit.get_mut(&name).unwrap();
                for skill_id in &skills {
                    let resolved = resolve_skills_for_bundle(skill_id, &registry)?;
                    for (src, fq) in resolved {
                        if !kit.skills.contains(&fq) {
                            kit.skills.push(fq);
                            added += 1;
                            if ext_sources.contains(src.as_str()) {
                                external += 1;
                            } else {
                                local += 1;
                            }
                        }
                    }
                }
            }

            let total = config.kit[&name].skills.len();
            crate::config::save(&config, config_path_str)?;
            if added > 0 {
                out.success(&format!("Created kit '{}'", name));
                out.info(&format!(
                    "  {} added ({}), {} total",
                    added,
                    source_breakdown(external, local),
                    total,
                ));
            } else {
                out.success(&format!("Created kit '{}'", name));
            }
            Ok(())
        }
        KitCommand::Delete { name, force } => {
            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            if !config.kit.contains_key(&name) {
                anyhow::bail!("kit '{}' not found", name);
            }

            let execute = force && !flags.dry_run;
            if execute {
                config.kit.remove(&name);
                crate::config::save(&config, config_path_str)?;
                out.success(&format!("Deleted kit '{}'", name));
            } else {
                out.info(&format!("Would delete kit '{}'", name));
                out.info("Use --force to delete");
            }
            Ok(())
        }
        KitCommand::List { patterns } => {
            let kits: Vec<(&String, &crate::config::KitConfig)> = if patterns.is_empty() {
                config.kit.iter().collect()
            } else {
                config
                    .kit
                    .iter()
                    .filter(|(name, _)| {
                        patterns.iter().any(|pat| {
                            if crate::registry::is_glob(pat) {
                                glob_match::glob_match(pat, name)
                            } else {
                                *name == pat || name.contains(pat.as_str())
                            }
                        })
                    })
                    .collect()
            };

            if flags.json {
                let entries: Vec<serde_json::Value> = kits
                    .iter()
                    .map(|(name, b)| {
                        serde_json::json!({
                            "name": name,
                            "skills": b.skills,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&entries)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            if kits.is_empty() {
                if patterns.is_empty() {
                    out.info("No kits configured. Use `equip kit create` to create one.");
                } else {
                    out.info("No kits matched the given pattern(s)");
                }
                return Ok(());
            }

            for (name, b) in &kits {
                println!(
                    "{} {}",
                    name.bold(),
                    format!("({})", b.skills.len()).dimmed()
                );
                for (i, skill_id) in b.skills.iter().enumerate() {
                    let connector = if i == b.skills.len() - 1 {
                        "└──"
                    } else {
                        "├──"
                    };
                    let display = if let Some((source, rest)) = skill_id.split_once(':') {
                        if let Some((plugin, skill)) = rest.split_once('/') {
                            crate::output::format_identity(source, plugin, skill)
                        } else {
                            skill_id.clone()
                        }
                    } else {
                        skill_id.clone()
                    };
                    println!("  {} {}", connector.dimmed(), display);
                }
            }
            Ok(())
        }
        KitCommand::Show { name } => {
            let kit = config
                .kit
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("kit '{}' not found", name))?;

            if flags.json {
                let json = serde_json::json!({
                    "name": name,
                    "skills": kit.skills,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            out.status("Kit", &name);
            out.status("Skills", &kit.skills.len().to_string());

            if !kit.skills.is_empty() {
                out.info("");
                let tree: Vec<(usize, String)> =
                    kit.skills.iter().map(|s| (0, s.clone())).collect();
                out.tree(&tree);
            }

            Ok(())
        }
        KitCommand::Add { name, skills } => {
            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            if !config.kit.contains_key(&name) {
                anyhow::bail!("kit '{}' not found", name);
            }

            let mut registry = crate::registry::load_registry(&data_dir)?;
            let renames =
                crate::registry::reconcile_with_config(&mut registry, &config.source, &data_dir)?;
            if !renames.is_empty() {
                crate::registry::save_registry(&registry, &data_dir)?;
                if !flags.quiet {
                    for r in &renames {
                        eprintln!("source renamed: {}", r);
                    }
                }
            }
            let ext_sources = external_source_set(&config);

            let kit = config.kit.get_mut(&name).unwrap();
            let mut added = 0usize;
            let mut external = 0usize;
            let mut local = 0usize;
            for skill_id in &skills {
                let resolved = resolve_skills_for_bundle(skill_id, &registry)?;
                for (src, fq) in resolved {
                    if !kit.skills.contains(&fq) {
                        kit.skills.push(fq);
                        added += 1;
                        if ext_sources.contains(src.as_str()) {
                            external += 1;
                        } else {
                            local += 1;
                        }
                    }
                }
            }

            let total = kit.skills.len();
            crate::config::save(&config, config_path_str)?;
            out.success(&format!(
                "Added {} skill(s) to kit '{}' ({}), {} total",
                added,
                name,
                source_breakdown(external, local),
                total,
            ));
            Ok(())
        }
        KitCommand::Drop { name, skills } => {
            let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            let kit = config
                .kit
                .get_mut(&name)
                .ok_or_else(|| anyhow::anyhow!("kit '{}' not found", name))?;

            let before = kit.skills.len();
            for skill_id in &skills {
                kit.skills.retain(|s| s != skill_id);
            }
            let dropped = before - kit.skills.len();
            let remaining = kit.skills.len();

            crate::config::save(&config, config_path_str)?;
            out.success(&format!(
                "Dropped {} skill(s) from kit '{}', {} remaining",
                dropped, name, remaining,
            ));
            Ok(())
        }
    }
}
