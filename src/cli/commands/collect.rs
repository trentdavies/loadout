use crate::cli::flags::Flags;
use crate::cli::helpers::{copy_dir_all, generate_marketplace};

pub(crate) fn run(
    agent: String,
    patterns: Vec<String>,
    adopt: bool,
    force: bool,
    interactive: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let config = crate::config::load(config_path_str)?;

    let data_dir = crate::config::data_dir();
    let mut registry = crate::registry::load_registry(&data_dir)?;
    let renames = crate::registry::reconcile_with_config(&mut registry, &config.source, &data_dir)?;
    if !renames.is_empty() {
        crate::registry::save_registry(&registry, &data_dir)?;
        if !flags.quiet {
            for rename in &renames {
                eprintln!("source reconciled: {}", rename);
            }
        }
    }
    let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);

    let agent_cfg = config
        .agent
        .iter()
        .find(|candidate| candidate.name == agent)
        .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", agent))?;
    let adapter = crate::agent::resolve_adapter(agent_cfg, &config.adapter)?;
    let installed_on_agent = adapter.installed_skills(&agent_cfg.path)?;
    let agent_installs = registry.installed.get(&agent).cloned().unwrap_or_default();

    let matched: Vec<String> = if patterns.is_empty() {
        installed_on_agent.clone()
    } else {
        installed_on_agent
            .iter()
            .filter(|skill_name| {
                patterns.iter().any(|pattern| {
                    let expanded = crate::registry::expand_pattern(pattern);
                    if glob_match::glob_match(&expanded, skill_name) {
                        return true;
                    }
                    if let Some(info) = agent_installs.get(*skill_name) {
                        let identity = format!("{}:{}/{}", info.source, info.plugin, info.skill);
                        glob_match::glob_match(&expanded, &identity)
                    } else {
                        false
                    }
                })
            })
            .cloned()
            .collect()
    };

    let mut tracked: Vec<(String, crate::registry::InstalledSkill)> = Vec::new();
    let mut untracked: Vec<String> = Vec::new();
    for skill_name in &matched {
        if let Some(info) = agent_installs.get(skill_name) {
            tracked.push((skill_name.clone(), info.clone()));
        } else {
            untracked.push(skill_name.clone());
        }
    }

    if matched.is_empty() {
        if patterns.is_empty() {
            out.info("No skills found on agent.");
        } else {
            out.info("No skills matched the given patterns.");
        }
        crate::registry::save_registry(&registry, &data_dir)?;
        return Ok(());
    }

    if !tracked.is_empty() {
        out.info("Tracked:");
        for (_name, info) in &tracked {
            let identity = crate::output::format_identity(&info.source, &info.plugin, &info.skill);
            out.info(&format!("  {} ← {}", identity, info.origin));
        }
    }
    if !untracked.is_empty() {
        out.info("Untracked:");
        for name in &untracked {
            out.info(&format!("  {}", name));
        }
    }

    let use_interactive = interactive || (patterns.is_empty() && crate::prompt::is_interactive());

    let (collect_tracked, adopt_untracked) = if force {
        (
            tracked
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>(),
            untracked.clone(),
        )
    } else if use_interactive {
        let all_labels: Vec<String> = tracked
            .iter()
            .map(|(name, info)| {
                format!("{} ({}:{}/{})", name, info.source, info.plugin, info.skill)
            })
            .chain(untracked.iter().map(|name| format!("{} (untracked)", name)))
            .collect();
        let label_refs: Vec<&str> = all_labels.iter().map(|label| label.as_str()).collect();
        let defaults: Vec<bool> = vec![true; all_labels.len()];

        let selected = crate::prompt::multi_select(
            "Select skills to collect",
            &label_refs,
            &defaults,
            flags.quiet,
        );

        let mut selected_tracked = Vec::new();
        let mut selected_untracked = Vec::new();
        for idx in selected {
            if idx < tracked.len() {
                selected_tracked.push(tracked[idx].0.clone());
            } else {
                selected_untracked.push(untracked[idx - tracked.len()].clone());
            }
        }

        if !selected_untracked.is_empty()
            && !crate::prompt::confirm_action(
                &format!(
                    "Adopt {} untracked skill(s) into local/?",
                    selected_untracked.len()
                ),
                flags.quiet,
                true,
            )
        {
            selected_untracked.clear();
        }

        (selected_tracked, selected_untracked)
    } else if !patterns.is_empty() {
        let selected_untracked = if !untracked.is_empty()
            && (adopt
                || crate::prompt::confirm_action(
                    &format!("Adopt {} untracked skill(s) into local/?", untracked.len()),
                    flags.quiet,
                    false,
                )) {
            untracked.clone()
        } else {
            Vec::new()
        };
        (
            tracked.iter().map(|(name, _)| name.clone()).collect(),
            selected_untracked,
        )
    } else {
        crate::registry::save_registry(&registry, &data_dir)?;
        return Ok(());
    };

    for skill_name in &collect_tracked {
        if let Some(info) = agent_installs.get(skill_name) {
            let agent_skill_dir = agent_cfg.path.join("skills").join(skill_name);
            let dest = data_dir.join(&info.origin);
            std::fs::create_dir_all(&dest)?;
            copy_dir_all(&agent_skill_dir, &dest)?;
            let identity = crate::output::format_identity(&info.source, &info.plugin, &info.skill);
            out.success(&format!("Collected {} → {}", identity, info.origin));
        }
    }

    if !adopt_untracked.is_empty() {
        let local_plugin = crate::config::plugins_dir().join("local");
        for name in &adopt_untracked {
            let agent_skill_dir = agent_cfg.path.join("skills").join(name);
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

    crate::registry::save_registry(&registry, &data_dir)?;
    Ok(())
}
