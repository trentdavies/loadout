use crate::cli::flags::Flags;
use crate::cli::helpers::{copy_dir_all, generate_marketplace, load_context, record_provenance_as};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UntrackedAction<'a> {
    Skip,
    AdoptLocal,
    Link(&'a str),
}

fn resolve_selection_patterns(
    config: &crate::config::Config,
    patterns: Vec<String>,
    kit: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let mut selection_patterns = patterns;
    if let Some(kit_name) = kit {
        let kit_cfg = config
            .kit
            .get(kit_name)
            .ok_or_else(|| anyhow::anyhow!("kit '{}' not found", kit_name))?;
        selection_patterns.extend(kit_cfg.skills.iter().cloned());
    }
    Ok(selection_patterns)
}

fn resolve_untracked_action<'a>(
    selected_untracked: &[String],
    adopt_local: bool,
    link: Option<&'a str>,
) -> anyhow::Result<UntrackedAction<'a>> {
    if adopt_local && link.is_some() {
        anyhow::bail!("--adopt-local cannot be combined with --link");
    }

    if let Some(identity) = link {
        if selected_untracked.is_empty() {
            anyhow::bail!("--link requires exactly one untracked installed skill");
        }
        if selected_untracked.len() > 1 {
            anyhow::bail!(
                "--link requires exactly one untracked installed skill; matched {}",
                selected_untracked.len()
            );
        }
        return Ok(UntrackedAction::Link(identity));
    }

    if adopt_local {
        return Ok(UntrackedAction::AdoptLocal);
    }

    Ok(UntrackedAction::Skip)
}

pub(crate) fn run(
    agent: String,
    patterns: Vec<String>,
    kit: Option<String>,
    link: Option<String>,
    adopt_local: bool,
    force: bool,
    interactive: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let ctx = load_context(flags)?;
    let config = ctx.config;
    let data_dir = ctx.data_dir;
    let mut registry = ctx.registry;
    let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);

    let agent_cfg = config
        .agent
        .iter()
        .find(|candidate| candidate.name == agent)
        .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", agent))?;
    let adapter = crate::agent::resolve_adapter(agent_cfg, &config.adapter)?;
    let installed_on_agent = adapter.installed_skills(&agent_cfg.path)?;
    let agent_installs = registry.installed.get(&agent).cloned().unwrap_or_default();

    let selection_patterns = resolve_selection_patterns(&config, patterns, kit.as_deref())?;

    let matched: Vec<String> = if selection_patterns.is_empty() {
        installed_on_agent.clone()
    } else {
        installed_on_agent
            .iter()
            .filter(|skill_name| {
                selection_patterns.iter().any(|pattern| {
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
        if selection_patterns.is_empty() {
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

    let auto_interactive = selection_patterns.is_empty()
        && !force
        && !adopt_local
        && link.is_none()
        && crate::prompt::is_interactive();
    let use_interactive = interactive || auto_interactive;

    let (collect_tracked, selected_untracked) = if use_interactive {
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

        (selected_tracked, selected_untracked)
    } else if force || !selection_patterns.is_empty() || adopt_local || link.is_some() {
        (
            tracked
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>(),
            untracked.clone(),
        )
    } else {
        crate::registry::save_registry(&registry, &data_dir)?;
        return Ok(());
    };

    let untracked_action =
        resolve_untracked_action(&selected_untracked, adopt_local, link.as_deref())?;

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

    match untracked_action {
        UntrackedAction::Skip if !selected_untracked.is_empty() => {
            out.warn(&format!(
                "Skipped {} untracked skill(s); use --adopt-local or --link <identity> to claim them.",
                selected_untracked.len()
            ));
        }
        UntrackedAction::Skip => {}
        UntrackedAction::AdoptLocal => {
            if selected_untracked.is_empty() {
                out.warn("No untracked skills selected for --adopt-local.");
            } else {
                let local_plugin = crate::config::plugins_dir().join("local");
                for name in &selected_untracked {
                    let agent_skill_dir = agent_cfg.path.join("skills").join(name);
                    let dest = local_plugin.join("skills").join(name);
                    std::fs::create_dir_all(&dest)?;
                    copy_dir_all(&agent_skill_dir, &dest)?;
                    let identity = crate::output::format_identity("local", "local", name);
                    out.success(&format!("Adopted {}", identity));

                    let agent_map = registry
                        .installed
                        .entry(agent_cfg.name.clone())
                        .or_default();
                    agent_map.insert(
                        name.clone(),
                        crate::registry::InstalledSkill {
                            source: "local".to_string(),
                            plugin: "local".to_string(),
                            skill: name.clone(),
                            origin: format!("local/skills/{}", name),
                        },
                    );
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
        UntrackedAction::Link(identity) => {
            let installed_name = &selected_untracked[0];
            let (source_name, plugin_name, skill) = {
                let (source_name, plugin, skill) = registry.find_skill_entry(identity)?;
                (source_name.to_string(), plugin.name.clone(), skill.clone())
            };
            let agent_skill_dir = agent_cfg.path.join("skills").join(installed_name);
            std::fs::create_dir_all(&skill.path)?;
            copy_dir_all(&agent_skill_dir, &skill.path)?;
            record_provenance_as(
                &mut registry,
                &data_dir,
                agent_cfg,
                installed_name,
                &source_name,
                &plugin_name,
                &skill,
            );
            out.success(&format!(
                "Linked {} → {}",
                installed_name,
                crate::output::format_identity(&source_name, &plugin_name, &skill.name)
            ));
        }
    }

    crate::registry::save_registry(&registry, &data_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_untracked_action, UntrackedAction};

    #[test]
    fn link_requires_exactly_one_untracked_skill() {
        let err = resolve_untracked_action(&[], false, Some("src:plug/skill")).unwrap_err();
        assert!(err
            .to_string()
            .contains("--link requires exactly one untracked installed skill"));

        let err = resolve_untracked_action(
            &["a".to_string(), "b".to_string()],
            false,
            Some("src:plug/skill"),
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("--link requires exactly one untracked installed skill"));
    }

    #[test]
    fn adopt_local_and_link_conflict() {
        let err =
            resolve_untracked_action(&["a".to_string()], true, Some("src:plug/skill")).unwrap_err();
        assert!(err
            .to_string()
            .contains("--adopt-local cannot be combined with --link"));
    }

    #[test]
    fn explicit_untracked_actions_resolve() {
        assert_eq!(
            resolve_untracked_action(&["a".to_string()], true, None).unwrap(),
            UntrackedAction::AdoptLocal
        );
        assert_eq!(
            resolve_untracked_action(&["a".to_string()], false, Some("src:plug/skill")).unwrap(),
            UntrackedAction::Link("src:plug/skill")
        );
        assert_eq!(
            resolve_untracked_action(&["a".to_string()], false, None).unwrap(),
            UntrackedAction::Skip
        );
    }
}
