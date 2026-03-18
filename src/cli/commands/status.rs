use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::load_context;

pub(crate) fn run(flags: &Flags) -> anyhow::Result<()> {
    let ctx = load_context(flags)?;
    let config = ctx.config;
    let registry = ctx.registry;
    let source_labels = ctx.source_labels;

    // Count installed skills across agents
    let mut total_installed = 0;
    for ac in &config.agent {
        let adapter = crate::agent::resolve_adapter(ac, &config.adapter).ok();
        if let Some(a) = adapter {
            if let Ok(skills) = a.installed_skills(&ac.path) {
                total_installed += skills.len();
            }
        }
    }

    let total_skills: usize = registry
        .sources
        .iter()
        .flat_map(|s| &s.plugins)
        .map(|p| p.skills.len())
        .sum();

    if flags.json {
        let json = serde_json::json!({
            "sources": registry.sources.len(),
            "agents": config.agent.len(),
            "plugins": registry.sources.iter().flat_map(|s| &s.plugins).count(),
            "skills": total_skills,
            "installed": total_installed,
            "kits": config.kit.len(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);

    // Sources section
    out.header("Sources");
    if registry.sources.is_empty() {
        out.info("  (none)");
    } else {
        let mut sources: Vec<&crate::registry::RegisteredSource> =
            registry.sources.iter().collect();
        sources.sort_by(|a, b| a.name.cmp(&b.name));

        for src in sources {
            let skill_count: usize = registry
                .sources
                .iter()
                .find(|rs| rs.name == src.name)
                .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                .unwrap_or(0);
            let config_src = config
                .source
                .iter()
                .find(|candidate| candidate.name == src.name);
            let version = config_src
                .and_then(|candidate| candidate.r#ref.as_deref())
                .unwrap_or("latest");
            let mode_str = config_src
                .and_then(|candidate| candidate.mode.as_deref())
                .unwrap_or("");
            let detail = if mode_str.is_empty() {
                format!(
                    "{} skills, @ {}, {}",
                    skill_count,
                    version,
                    src.residence.as_str()
                )
            } else {
                format!(
                    "{} skills, @ {}, {}, {}",
                    skill_count,
                    version,
                    src.residence.as_str(),
                    mode_str
                )
            };
            let label = source_labels
                .get(&src.name)
                .map(String::as_str)
                .unwrap_or(src.name.as_str());
            println!("  {} {}", label.bold(), detail.dimmed(),);
        }
    }

    // Targets section
    out.header("Agents");
    if config.agent.is_empty() {
        out.info("  (none)");
    } else {
        for ac in &config.agent {
            let adapter = crate::agent::resolve_adapter(ac, &config.adapter).ok();
            let installed_count = adapter
                .as_ref()
                .and_then(|a| a.installed_skills(&ac.path).ok())
                .map(|s| s.len())
                .unwrap_or(0);
            println!(
                "  {} {} {}",
                ac.name.bold(),
                format!("({})", ac.agent_type).cyan(),
                format!(
                    "{} installed, scope: {}, sync: {}",
                    installed_count, ac.scope, ac.sync
                )
                .dimmed(),
            );
        }
    }

    // Kits section
    out.header("Kits");
    if config.kit.is_empty() {
        out.info("  (none)");
    } else {
        for (name, kit) in &config.kit {
            println!(
                "  {} {}",
                name.bold(),
                format!("({} skills)", kit.skills.len()).dimmed(),
            );
        }
    }

    // Summary
    out.info("");
    out.status(
        "Total",
        &format!(
            "{} sources, {} plugins, {} skills, {} installed",
            registry.sources.len(),
            registry.sources.iter().flat_map(|s| &s.plugins).count(),
            total_skills,
            total_installed,
        ),
    );

    Ok(())
}
