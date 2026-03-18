use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{extract_domain, load_context, resolve_skill_patterns};

enum AddedSourceSummary {
    External { source_name: String },
    Local(crate::source::LocalImport),
}

struct StagedParsedSource {
    parsed: crate::source::ParsedSource,
    _temp_dir: Option<tempfile::TempDir>,
}

pub(crate) fn run_add(
    url: String,
    source: Option<String>,
    plugin: Option<String>,
    skill: Option<String>,
    name: Option<String>,
    r#ref: Option<String>,
    symlink: bool,
    copy: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    // Backward-compat: error on deprecated --name
    if name.is_some() {
        anyhow::bail!("`--name` has been renamed to `--source`");
    }

    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    let source_url = crate::source::SourceUrl::parse(&url)?;
    let effective_ref = r#ref.as_deref().or_else(|| source_url.tree_ref());
    let staged = stage_source_for_parse(&source_url, effective_ref, &data_dir)?;
    let residence = crate::source::source_kind_residence(staged.parsed.kind);

    let source_name = match residence {
        crate::config::SourceResidence::External => {
            let default_source = source_url.default_name();
            let source_name = if let Some(s) = source.clone() {
                s
            } else {
                crate::prompt::confirm_or_override("Source name", &default_source, flags.quiet)
            };

            if source_name == "local" {
                anyhow::bail!(
                    "'local' is reserved for the local plugin source. Use --source to choose a different name."
                );
            }

            if config.source.iter().any(|existing| existing.name == source_name) {
                anyhow::bail!(
                    "source '{}' already exists. Use --source to choose a different alias.",
                    source_name
                );
            }

            source_name
        }
        crate::config::SourceResidence::Local => "local".to_string(),
    };

    let overrides = prompt_add_overrides(
        &staged.parsed,
        source.clone(),
        plugin,
        skill,
        residence,
        flags,
    );
    let norm_overrides = crate::source::normalize::Overrides {
        plugin: overrides.0.as_deref(),
        skill: overrides.1.as_deref(),
    };

    let added = if flags.dry_run {
        None
    } else {
        Some(match residence {
            crate::config::SourceResidence::External => {
                let cache_path = crate::source::source_storage_path(&source_name, residence);
                let use_symlink = match &source_url {
                    crate::source::SourceUrl::Local(path) if path.is_dir() => {
                        if symlink {
                            true
                        } else if copy {
                            false
                        } else {
                            crate::prompt::prompt_fetch_mode(flags.quiet) == "symlink"
                        }
                    }
                    _ => false,
                };

                if !flags.quiet {
                    let action = match &source_url {
                        crate::source::SourceUrl::Git(url, _) => format!("Cloning {}", url.dimmed()),
                        crate::source::SourceUrl::Local(path) if use_symlink => {
                            format!("Linking {}", path.display().to_string().dimmed())
                        }
                        crate::source::SourceUrl::Local(path) => {
                            format!("Copying {}", path.display().to_string().dimmed())
                        }
                        crate::source::SourceUrl::Archive(path) => {
                            format!("Extracting {}", path.display().to_string().dimmed())
                        }
                    };
                    eprintln!("{}", action);
                }

                crate::source::fetch::fetch_with_mode(
                    &source_url,
                    &cache_path,
                    effective_ref,
                    use_symlink,
                )?;

                let prepared = crate::source::prepare_source(
                    &source_name,
                    &source_url,
                    &cache_path,
                    r#ref.clone(),
                    if use_symlink {
                        Some("symlink".to_string())
                    } else {
                        None
                    },
                    residence,
                    &norm_overrides,
                )?;

                if !flags.quiet && !crate::prompt::is_interactive() {
                    for plugin in &prepared.registered.plugins {
                        for skill in &plugin.skills {
                            eprintln!(
                                "resolved: {}",
                                crate::output::plain_identity(
                                    &source_name,
                                    &plugin.name,
                                    &skill.name
                                )
                            );
                        }
                    }
                }

                let mut registry = crate::registry::load_registry(&data_dir)?;
                crate::source::persist_prepared_source(&mut config, &mut registry, prepared);
                crate::registry::save_registry(&registry, &data_dir)?;
                crate::config::save(&config, config_path_str)?;

                AddedSourceSummary::External { source_name }
            }
            crate::config::SourceResidence::Local => {
                let imported =
                    crate::source::import_into_local_source(&staged.parsed, &norm_overrides, &data_dir)?;

                if !flags.quiet && !crate::prompt::is_interactive() {
                    for plugin in &imported.plugins {
                        for skill in &plugin.skills {
                            eprintln!(
                                "resolved: {}",
                                crate::output::plain_identity(
                                    &imported.source_name,
                                    &plugin.name,
                                    &skill.name
                                )
                            );
                        }
                    }
                }

                let registry =
                    crate::cli::helpers::load_effective_registry(&config, &data_dir, flags.quiet)?;
                crate::registry::save_registry(&registry, &data_dir)?;

                AddedSourceSummary::Local(imported)
            }
        })
    };

    if !flags.quiet {
        match added {
            Some(AddedSourceSummary::External { source_name }) => {
                let reg = crate::registry::load_registry(&data_dir)?;
                if let Some(src) = reg.sources.iter().find(|s| s.name == source_name) {
                    print_add_summary(
                        &format!("Added source {}", source_name.bold()),
                        &src.plugins,
                        r#ref.as_deref(),
                        flags.verbose,
                    );
                } else {
                    println!("{} Added source {}", "✓".green(), source_name.bold());
                }
            }
            Some(AddedSourceSummary::Local(imported)) => {
                let label = imported.display_name.as_deref().unwrap_or("local");
                let heading = if label == "local" {
                    "Imported into local source".to_string()
                } else {
                    format!("Imported into local source {}", format!("({label})").dimmed())
                };
                print_add_summary(&heading, &imported.plugins, None, flags.verbose);
            }
            None => {}
        }
    }
    Ok(())
}

fn stage_source_for_parse(
    source_url: &crate::source::SourceUrl,
    git_ref: Option<&str>,
    data_dir: &std::path::Path,
) -> anyhow::Result<StagedParsedSource> {
    match source_url {
        crate::source::SourceUrl::Local(path) => Ok(StagedParsedSource {
            parsed: crate::source::ParsedSource::parse(&crate::source::detect_path(source_url, path))?,
            _temp_dir: None,
        }),
        crate::source::SourceUrl::Git(..) | crate::source::SourceUrl::Archive(..) => {
            let temp_dir = tempfile::Builder::new()
                .prefix("equip-add-")
                .tempdir_in(crate::config::internal_dir().parent().unwrap_or(data_dir))?;
            let staged_root = temp_dir.path().join("source");
            crate::source::fetch::fetch(source_url, &staged_root, git_ref)?;
            Ok(StagedParsedSource {
                parsed: crate::source::ParsedSource::parse(&crate::source::detect_path(
                    source_url,
                    &staged_root,
                ))?,
                _temp_dir: Some(temp_dir),
            })
        }
    }
}

fn prompt_add_overrides(
    parsed: &crate::source::ParsedSource,
    source: Option<String>,
    plugin: Option<String>,
    skill: Option<String>,
    residence: crate::config::SourceResidence,
    flags: &Flags,
) -> (Option<String>, Option<String>) {
    let source_as_plugin = if residence == crate::config::SourceResidence::Local {
        source
    } else {
        None
    };

    let plugin_override = if let Some(plugin) = plugin {
        Some(plugin)
    } else if let Some(source_plugin) = source_as_plugin {
        Some(source_plugin)
    } else if let Some(default_plugin) = parsed.prompt_plugin_name() {
        let confirmed = crate::prompt::confirm_or_override("Plugin name", default_plugin, flags.quiet);
        if confirmed != default_plugin {
            Some(confirmed)
        } else {
            None
        }
    } else {
        None
    };

    let skill_override = if let Some(skill) = skill {
        Some(skill)
    } else if let Some(default_skill) = parsed.prompt_skill_name() {
        let confirmed = crate::prompt::confirm_or_override("Skill name", default_skill, flags.quiet);
        if confirmed != default_skill {
            Some(confirmed)
        } else {
            None
        }
    } else {
        None
    };

    (plugin_override, skill_override)
}

fn print_add_summary(
    heading: &str,
    plugins: &[crate::registry::RegisteredPlugin],
    git_ref: Option<&str>,
    verbose: bool,
) {
    let plugin_count = plugins.len();
    let skill_count: usize = plugins.iter().map(|plugin| plugin.skills.len()).sum();

    println!(
        "{} {} {} {}",
        "✓".green(),
        heading,
        format!(
            "({} plugin{}, {} skill{})",
            plugin_count,
            if plugin_count == 1 { "" } else { "s" },
            skill_count,
            if skill_count == 1 { "" } else { "s" },
        )
        .dimmed(),
        git_ref
            .map(|value| format!("@ {}", value.cyan()))
            .unwrap_or_default(),
    );

    if !verbose {
        return;
    }

    for plugin in plugins {
        println!("  {} {}", "├──".dimmed(), plugin.name.green());
        for (index, skill) in plugin.skills.iter().enumerate() {
            let connector = if index == plugin.skills.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            let desc = skill.description.as_deref().unwrap_or("");
            if desc.is_empty() {
                println!("  {}   {} {}", "│".dimmed(), connector.dimmed(), skill.name);
            } else {
                println!(
                    "  {}   {} {} {}",
                    "│".dimmed(),
                    connector.dimmed(),
                    skill.name,
                    format!("— {}", desc).dimmed(),
                );
            }
        }
    }
}

pub(crate) fn run_list(
    patterns: Vec<String>,
    external: bool,
    fzf: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let ctx = load_context(flags)?;
    let config_for_list = ctx.config;
    let registry = ctx.registry;

    if external {
        // List external sources in table format
        if flags.json {
            let entries: Vec<serde_json::Value> = config_for_list
                .source
                .iter()
                .map(|src| {
                    let skill_count: usize = registry
                        .sources
                        .iter()
                        .find(|rs| rs.name == src.name)
                        .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                        .unwrap_or(0);
                    serde_json::json!({
                        "name": src.name,
                        "type": src.source_type,
                        "residence": src.residence.as_str(),
                        "domain": extract_domain(&src.url),
                        "ref": src.r#ref,
                        "skills": skill_count,
                        "mode": src.mode,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&entries)?);
            return Ok(());
        }

        if config_for_list.source.is_empty() {
            let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            output.info("No sources configured. Use `equip add` to add one.");
            return Ok(());
        }

        let rows: Vec<Vec<String>> = config_for_list
            .source
            .iter()
            .map(|src| {
                let skill_count: usize = registry
                    .sources
                    .iter()
                    .find(|rs| rs.name == src.name)
                    .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                    .unwrap_or(0);
                vec![
                    src.name.clone(),
                    src.source_type.clone(),
                    src.residence.as_str().to_string(),
                    extract_domain(&src.url),
                    src.r#ref.clone().unwrap_or_default(),
                    skill_count.to_string(),
                    src.mode.clone().unwrap_or_default(),
                ]
            })
            .collect();

        let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        out.table(
            &["NAME", "TYPE", "STORAGE", "DOMAIN", "REF", "SKILLS", "MODE"],
            &rows,
        );
        return Ok(());
    }

    // Collect matching skills from patterns
    let skills: Vec<(
        &str,
        &crate::registry::RegisteredPlugin,
        &crate::registry::RegisteredSkill,
    )> = if patterns.is_empty() {
        registry.all_skills()
    } else {
        resolve_skill_patterns(&patterns, &registry, true)?
    };

    // Interactive fzf mode
    if fzf {
        if skills.is_empty() {
            let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            output.info("No skills to browse.");
            return Ok(());
        }

        let mut lines = Vec::new();
        for (source_name, plugin, skill) in &skills {
            let identity = crate::output::plain_identity(source_name, &plugin.name, &skill.name);
            let skill_md = skill.path.join("SKILL.md");
            lines.push(format!("{}\t{}", identity, skill_md.display()));
        }

        let input = lines.join("\n");

        let mut child = std::process::Command::new("fzf")
            .args([
                "--ansi",
                "--delimiter=\t",
                "--with-nth=1",
                "--preview=cat {2}",
                "--preview-window=right:60%:wrap",
                "--header=Skills (tab to preview)",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!(
                        "fzf not found in PATH. Install fzf: https://github.com/junegunn/fzf"
                    )
                } else {
                    anyhow::anyhow!("failed to spawn fzf: {}", e)
                }
            })?;

        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            let _ = stdin.write_all(input.as_bytes());
        }
        drop(child.stdin.take());

        let output = child.wait_with_output()?;
        if output.status.success() {
            let selected = String::from_utf8_lossy(&output.stdout);
            let selected = selected.trim();
            if let Some(identity) = selected.split('\t').next() {
                println!("{}", identity);
            }
        }
        return Ok(());
    }

    // Single result → show detail view
    if skills.len() == 1 {
        let (source_name, plugin, skill) = skills[0];
        let plugin_name = &plugin.name;

        if flags.json {
            let json = serde_json::json!({
                "identity": crate::output::plain_identity(source_name, plugin_name, &skill.name),
                "name": skill.name,
                "plugin": plugin_name,
                "source": source_name,
                "description": skill.description,
                "path": skill.path,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            return Ok(());
        }

        let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        out.status(
            "Identity",
            &crate::output::format_identity(source_name, plugin_name, &skill.name),
        );
        out.status(
            "Description",
            skill.description.as_deref().unwrap_or("(none)"),
        );
        if flags.verbose {
            out.status("Path", &skill.path.display().to_string());
        }
    } else if flags.json {
        let entries: Vec<serde_json::Value> = skills.iter()
            .map(|(source_name, plugin, skill)| {
                let source_ref = config_for_list.source.iter()
                    .find(|cs| cs.name == *source_name)
                    .and_then(|cs| cs.r#ref.clone());
                let mut entry = serde_json::json!({
                    "identity": crate::output::plain_identity(source_name, &plugin.name, &skill.name),
                    "name": skill.name,
                    "plugin": plugin.name,
                    "source": source_name,
                });
                if let Some(ref r) = source_ref {
                    entry["ref"] = serde_json::Value::String(r.clone());
                }
                entry
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        if skills.is_empty() {
            if patterns.is_empty() {
                output.info("No skills found. Add a source with `equip add`");
            } else {
                output.info("No skills matched the given pattern(s)");
            }
        } else {
            for (source_name, plugin, skill) in &skills {
                println!(
                    "{}",
                    crate::output::format_identity(source_name, &plugin.name, &skill.name)
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn run_remove(name: Option<String>, force: bool, flags: &Flags) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    let name = match name {
        Some(n) => n,
        None => {
            let source_names: Vec<String> = config.source.iter().map(|s| s.name.clone()).collect();
            if source_names.is_empty() {
                anyhow::bail!("no sources configured");
            }
            crate::prompt::select_from("Select source to remove", &source_names, flags.quiet)?
        }
    };

    if !config.source.iter().any(|s| s.name == name) {
        anyhow::bail!("source '{}' not found", name);
    }

    // Check if any skills from this source are installed on agents
    let registry = crate::registry::load_registry(&data_dir)?;
    let mut installed_on: Vec<String> = Vec::new();
    if let Some(reg_src) = registry.sources.iter().find(|s| s.name == name) {
        let skill_names: Vec<&str> = reg_src
            .plugins
            .iter()
            .flat_map(|p| p.skills.iter().map(|s| s.name.as_str()))
            .collect();
        for ac in &config.agent {
            let agent_path = std::path::PathBuf::from(&ac.path);
            if let Ok(adapter) = crate::agent::resolve_adapter(ac, &config.adapter) {
                if let Ok(installed) = adapter.installed_skills(&agent_path) {
                    for sk in &skill_names {
                        if installed.contains(&sk.to_string()) {
                            installed_on.push(ac.name.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    if !installed_on.is_empty() && !flags.quiet {
        eprintln!(
            "warning: source '{}' has installed skills on: {}",
            name,
            installed_on.join(", ")
        );
    }

    let execute = force && !flags.dry_run;
    if execute {
        // Remove cached content
        let residence = config
            .source
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.residence)
            .unwrap_or(crate::source::default_source_residence());
        let cache_path = crate::source::source_storage_path(&name, residence);
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)?;
        }

        // Remove from registry
        let mut registry = crate::registry::load_registry(&data_dir)?;
        registry.sources.retain(|s| s.name != name);
        crate::registry::save_registry(&registry, &data_dir)?;

        // Remove from config
        config.source.retain(|s| s.name != name);
        crate::config::save(&config, config_path_str)?;
    }

    if !flags.quiet {
        if execute {
            println!("Removed source '{}'", name);
        } else {
            println!("Would remove source '{}'", name);
            println!("Use --force to remove");
        }
    }
    Ok(())
}

pub(crate) fn run_update(
    name: Option<String>,
    update_ref: Option<String>,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();
    let mut registry = crate::registry::load_registry(&data_dir)?;
    let renames = crate::registry::reconcile_with_config(&mut registry, &config.source, &data_dir)?;
    if !renames.is_empty() {
        crate::registry::save_registry(&registry, &data_dir)?;
        if !flags.quiet {
            for r in &renames {
                eprintln!("source reconciled: {}", r);
            }
        }
    }

    if update_ref.is_some() && name.is_none() {
        anyhow::bail!("--ref requires a source name (e.g., equip update my-source --ref v2.0)");
    }

    // Determine which sources to update
    let sources_to_update: Vec<crate::config::SourceConfig> = if let Some(ref n) = name {
        let src = config
            .source
            .iter()
            .find(|s| s.name == *n)
            .ok_or_else(|| anyhow::anyhow!("source '{}' not found", n))?;
        vec![src.clone()]
    } else {
        if config.source.is_empty() {
            if !flags.quiet {
                println!("No sources to update.");
            }
            return Ok(());
        }
        config.source.clone()
    };

    let mut updated_registry = registry;
    let mut updated_count = 0;
    let mut errors = Vec::new();
    let mut ref_changed = false;

    for src in &sources_to_update {
        if !flags.quiet {
            println!("Updating '{}'...", src.name);
        }

        if flags.dry_run {
            if !flags.quiet {
                println!("  (dry run) would re-fetch from {}", src.url);
            }
            updated_count += 1;
            continue;
        }

        let cache_path = crate::source::source_storage_path_for_config(src);
        if src.mode.as_deref() == Some("symlink") && !flags.quiet {
            println!("  (symlinked, re-detecting)");
        }

        match crate::source::refresh_source(src, &cache_path, update_ref.as_deref()) {
            Ok(crate::source::RefreshSource::Updated(prepared)) => {
                if update_ref.is_some()
                    && prepared.config.r#ref != src.r#ref
                    && prepared.config.source_type == "git"
                {
                    ref_changed = true;
                }
                crate::source::persist_prepared_source(
                    &mut config,
                    &mut updated_registry,
                    prepared,
                );
                updated_count += 1;
            }
            Ok(crate::source::RefreshSource::SkippedPinned { pinned_ref }) => {
                if !flags.quiet {
                    eprintln!(
                        "warning: source '{}' is pinned to {}, skipping",
                        src.name, pinned_ref
                    );
                }
                continue;
            }
            Err(e) => {
                errors.push(format!("{}: {}", src.name, e));
                continue;
            }
        }
    }

    if !flags.dry_run {
        crate::registry::save_registry(&updated_registry, &data_dir)?;
        if ref_changed {
            if let Some(ref new_ref) = update_ref {
                if let Some(ref source_name) = name {
                    if let Some(cfg_src) = config.source.iter_mut().find(|s| s.name == *source_name)
                    {
                        cfg_src.r#ref = if new_ref == "latest" {
                            None
                        } else {
                            Some(new_ref.clone())
                        };
                    }
                }
            }
            crate::config::save(&config, config_path_str)?;
        }
    }

    if !flags.quiet {
        if updated_count > 0 {
            println!("Updated {} source(s)", updated_count);
        }
        for err in &errors {
            eprintln!("warning: {}", err);
        }
    }

    if !errors.is_empty() && updated_count == 0 {
        anyhow::bail!("all updates failed");
    }

    Ok(())
}
