use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::extract_domain;

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
    let default_source = source_url.default_name();
    let source_name = if let Some(s) = source {
        s
    } else {
        crate::prompt::confirm_or_override("Source name", &default_source, flags.quiet)
    };

    if config.source.iter().any(|s| s.name == source_name) {
        anyhow::bail!(
            "source '{}' already exists. Use --source to choose a different alias.",
            source_name
        );
    }

    let cache_path = crate::config::cache_dir().join(&source_name);

    // Determine fetch mode for local directory sources
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

    if !flags.dry_run {
        // Use tree ref from URL when no explicit --ref provided
        let effective_ref = r#ref.as_deref().or_else(|| source_url.tree_ref());
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

        // Detect on subpath within the clone if the URL points into a tree
        let detect_path = if let Some(subpath) = source_url.subpath() {
            cache_path.join(subpath)
        } else {
            cache_path.clone()
        };
        let structure = crate::source::detect::detect(&detect_path)?;

        // Determine default plugin/skill names from structure for prompting
        let overrides = {
            use crate::source::detect::SourceStructure;

            let plugin_override: Option<String> = if plugin.is_some() {
                plugin
            } else {
                let default_plugin = match &structure {
                    SourceStructure::SingleFile { .. }
                    | SourceStructure::SingleSkillDir { .. } => None,
                    SourceStructure::FlatSkills => {
                        let dir = detect_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.strip_prefix('.').unwrap_or(n))
                            .unwrap_or(&source_name);
                        if dir == source_name {
                            None
                        } else {
                            Some(dir.to_string())
                        }
                    }
                    SourceStructure::SinglePlugin => {
                        let plugin_json = detect_path.join(".claude-plugin/plugin.json");
                        if plugin_json.exists() {
                            let m = crate::source::manifest::load_plugin_manifest(
                                &plugin_json,
                            )?;
                            if m.name == source_name {
                                None
                            } else {
                                Some(m.name)
                            }
                        } else {
                            let n = detect_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unnamed")
                                .to_string();
                            if n == source_name {
                                None
                            } else {
                                Some(n)
                            }
                        }
                    }
                    SourceStructure::Marketplace => None,
                };
                if let Some(ref dp) = default_plugin {
                    let confirmed =
                        crate::prompt::confirm_or_override("Plugin name", dp, flags.quiet);
                    if confirmed != *dp {
                        Some(confirmed)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let skill_override: Option<String> = if skill.is_some() {
                skill
            } else {
                match &structure {
                    SourceStructure::SingleFile { skill_name } => {
                        let confirmed = crate::prompt::confirm_or_override(
                            "Skill name",
                            skill_name,
                            flags.quiet,
                        );
                        if confirmed != *skill_name {
                            Some(confirmed)
                        } else {
                            None
                        }
                    }
                    SourceStructure::SingleSkillDir { skill_name } => {
                        let confirmed = crate::prompt::confirm_or_override(
                            "Skill name",
                            skill_name,
                            flags.quiet,
                        );
                        if confirmed != *skill_name {
                            Some(confirmed)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            (plugin_override, skill_override)
        };

        let norm_overrides = crate::source::normalize::Overrides {
            plugin: overrides.0.as_deref(),
            skill: overrides.1.as_deref(),
        };

        let mut registered = crate::source::normalize::normalize_with(
            &source_name,
            &detect_path,
            &structure,
            &norm_overrides,
        )?;
        registered.url = source_url.url_string();

        // In non-interactive/quiet mode, show what was resolved
        if !flags.quiet && !crate::prompt::is_interactive() {
            for p in &registered.plugins {
                for s in &p.skills {
                    eprintln!(
                        "resolved: {}",
                        crate::output::plain_identity(&source_name, &p.name, &s.name)
                    );
                }
            }
        }

        let mut registry = crate::registry::load_registry(&data_dir)?;
        registry.sources.retain(|s| s.name != source_name);
        registry.sources.push(registered);
        crate::registry::save_registry(&registry, &data_dir)?;

        config.source.push(crate::config::SourceConfig {
            name: source_name.clone(),
            url: source_url.url_string(),
            source_type: source_url.source_type().to_string(),
            r#ref: r#ref.clone(),
            mode: if use_symlink {
                Some("symlink".to_string())
            } else {
                None
            },
        });
        crate::config::save(&config, config_path_str)?;
    }

    if !flags.quiet {
        // Load the registered source back to get plugin/skill counts
        let data_dir = crate::config::data_dir();
        let reg = crate::registry::load_registry(&data_dir)?;
        if let Some(src) = reg.sources.iter().find(|s| s.name == source_name) {
            let plugin_count = src.plugins.len();
            let skill_count: usize = src.plugins.iter().map(|p| p.skills.len()).sum();

            println!(
                "{} Added source {} {} {}",
                "✓".green(),
                source_name.bold(),
                format!("({} plugin{}, {} skill{})",
                    plugin_count,
                    if plugin_count == 1 { "" } else { "s" },
                    skill_count,
                    if skill_count == 1 { "" } else { "s" },
                ).dimmed(),
                if let Some(r) = &r#ref {
                    format!("@ {}", r.cyan())
                } else {
                    String::new()
                },
            );

            if flags.verbose {
                for p in &src.plugins {
                    println!("  {} {}", "├──".dimmed(), p.name.green());
                    for (i, s) in p.skills.iter().enumerate() {
                        let connector = if i == p.skills.len() - 1 { "└──" } else { "├──" };
                        let desc = s.description.as_deref().unwrap_or("");
                        if desc.is_empty() {
                            println!("  {}   {} {}", "│".dimmed(), connector.dimmed(), s.name);
                        } else {
                            println!(
                                "  {}   {} {} {}",
                                "│".dimmed(),
                                connector.dimmed(),
                                s.name,
                                format!("— {}", desc).dimmed(),
                            );
                        }
                    }
                }
            }
        } else {
            println!("{} Added source {}", "✓".green(), source_name.bold());
        }
    }
    Ok(())
}

pub(crate) fn run_list(
    patterns: Vec<String>,
    external: bool,
    fzf: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let data_dir = crate::config::data_dir();
    let config_for_list = crate::config::load(flags.config_path())?;
    let mut registry = crate::registry::load_registry(&data_dir)?;
    let renames = crate::registry::reconcile_with_config(
        &mut registry,
        &config_for_list.source,
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
            let output =
                crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
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
                    extract_domain(&src.url),
                    src.r#ref.clone().unwrap_or_default(),
                    skill_count.to_string(),
                    src.mode.clone().unwrap_or_default(),
                ]
            })
            .collect();

        let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        out.table(&["NAME", "TYPE", "DOMAIN", "REF", "SKILLS", "MODE"], &rows);
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
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for pat in &patterns {
            if crate::registry::is_glob(pat) {
                for triple in registry.match_skills(pat) {
                    let id = crate::output::plain_identity(
                        triple.0,
                        &triple.1.name,
                        &triple.2.name,
                    );
                    if seen.insert(id) {
                        result.push(triple);
                    }
                }
            } else {
                match registry.find_skill(pat) {
                    Ok((src, plug, sk)) => {
                        let id = crate::output::plain_identity(src, plug, &sk.name);
                        if seen.insert(id) {
                            result.push((
                                src,
                                registry
                                    .sources
                                    .iter()
                                    .flat_map(|s| s.plugins.iter())
                                    .find(|p| p.name == plug)
                                    .unwrap(),
                                sk,
                            ));
                        }
                    }
                    Err(_) => {
                        for triple in registry.match_skills(pat) {
                            let id = crate::output::plain_identity(
                                triple.0,
                                &triple.1.name,
                                &triple.2.name,
                            );
                            if seen.insert(id) {
                                result.push(triple);
                            }
                        }
                    }
                }
            }
        }
        result
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
                    anyhow::anyhow!("fzf not found in PATH. Install fzf: https://github.com/junegunn/fzf")
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

pub(crate) fn run_remove(
    name: Option<String>,
    force: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    let name = match name {
        Some(n) => n,
        None => {
            let source_names: Vec<String> =
                config.source.iter().map(|s| s.name.clone()).collect();
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
        let cache_path = crate::config::cache_dir().join(&name);
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

    if update_ref.is_some() && name.is_none() {
        anyhow::bail!(
            "--ref requires a source name (e.g., equip update my-source --ref v2.0)"
        );
    }

    // Determine which sources to update
    let sources_to_update: Vec<&crate::config::SourceConfig> = if let Some(ref n) = name {
        let src = config
            .source
            .iter()
            .find(|s| s.name == *n)
            .ok_or_else(|| anyhow::anyhow!("source '{}' not found", n))?;
        vec![src]
    } else {
        if config.source.is_empty() {
            if !flags.quiet {
                println!("No sources to update.");
            }
            return Ok(());
        }
        config.source.iter().collect()
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

        let cache_path = crate::config::cache_dir().join(&src.name);

        let source_url = match crate::source::SourceUrl::parse(&src.url) {
            Ok(u) => u,
            Err(e) => {
                errors.push(format!("{}: {}", src.name, e));
                continue;
            }
        };

        let is_symlinked = src.mode.as_deref() == Some("symlink");
        match &source_url {
            crate::source::SourceUrl::Local(path) => {
                if is_symlinked {
                    if !flags.quiet {
                        println!("  (symlinked, re-detecting)");
                    }
                } else {
                    if cache_path.exists() {
                        std::fs::remove_dir_all(&cache_path)?;
                    }
                    if let Err(e) =
                        crate::source::fetch::fetch(&source_url, &cache_path, None)
                    {
                        errors.push(format!("{}: {}", src.name, e));
                        continue;
                    }
                }
                let _ = path;
            }
            crate::source::SourceUrl::Git(..) => {
                if let Some(ref new_ref) = update_ref {
                    if !cache_path.exists() {
                        let effective_ref = if new_ref == "latest" {
                            None
                        } else {
                            Some(new_ref.as_str())
                        };
                        if let Err(e) = crate::source::fetch::fetch(
                            &source_url,
                            &cache_path,
                            effective_ref,
                        ) {
                            errors.push(format!("{}: {}", src.name, e));
                            continue;
                        }
                    } else if new_ref == "latest" {
                        if let Err(e) = crate::source::fetch::update_git(&cache_path, None)
                        {
                            errors.push(format!("{}: {}", src.name, e));
                            continue;
                        }
                    } else if let Err(e) =
                        crate::source::fetch::switch_ref(&cache_path, new_ref)
                    {
                        errors.push(format!("{}: {}", src.name, e));
                        continue;
                    }
                    ref_changed = true;
                } else if cache_path.exists() {
                    match crate::source::fetch::update_git_ref(
                        &cache_path,
                        src.r#ref.as_deref(),
                    ) {
                        Ok(None) => {
                            if !flags.quiet {
                                let tag = src.r#ref.as_deref().unwrap_or("unknown");
                                eprintln!(
                                    "warning: source '{}' is pinned to {}, skipping",
                                    src.name, tag
                                );
                            }
                            continue;
                        }
                        Ok(Some(_)) => {}
                        Err(e) => {
                            errors.push(format!("{}: {}", src.name, e));
                            continue;
                        }
                    }
                } else if let Err(e) = crate::source::fetch::fetch(
                    &source_url,
                    &cache_path,
                    src.r#ref.as_deref(),
                ) {
                    errors.push(format!("{}: {}", src.name, e));
                    continue;
                }
            }
            crate::source::SourceUrl::Archive(_) => {
                if cache_path.exists() {
                    std::fs::remove_dir_all(&cache_path)?;
                }
                if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path, None)
                {
                    errors.push(format!("{}: {}", src.name, e));
                    continue;
                }
            }
        }

        // Re-detect and re-normalize
        let structure = match crate::source::detect::detect(&cache_path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("{}: detection failed: {}", src.name, e));
                continue;
            }
        };

        match crate::source::normalize::normalize(&src.name, &cache_path, &structure) {
            Ok(mut registered) => {
                registered.url = src.url.clone();
                updated_registry.sources.retain(|s| s.name != src.name);
                updated_registry.sources.push(registered);
                updated_count += 1;
            }
            Err(e) => {
                errors.push(format!("{}: normalization failed: {}", src.name, e));
            }
        }
    }

    if !flags.dry_run {
        crate::registry::save_registry(&updated_registry, &data_dir)?;
        if ref_changed {
            if let Some(ref new_ref) = update_ref {
                if let Some(ref source_name) = name {
                    if let Some(cfg_src) =
                        config.source.iter_mut().find(|s| s.name == *source_name)
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
