use crate::cli::flags::Flags;
use crate::cli::helpers::add_detected_agents;

pub(crate) fn run(url: Option<String>, flags: &Flags) -> anyhow::Result<()> {
    let path = crate::config::config_path(flags.config_path());
    if path.exists() {
        if url.is_some() && !flags.quiet {
            println!(
                "Config already exists at {}. Use `equip add` instead.",
                path.display()
            );
        } else if !flags.quiet {
            println!(
                "Config already exists at {}. Use `equip config edit` to modify.",
                path.display()
            );
        }
        return Ok(());
    }

    // Create directory structure
    let data = crate::config::data_dir();
    std::fs::create_dir_all(&data)?;
    std::fs::create_dir_all(crate::config::cache_dir())?;
    std::fs::create_dir_all(crate::config::internal_dir())?;

    // Legacy migration: rename sources/ to external/
    let legacy_sources = data.join("sources");
    let external_dir = data.join("external");
    if legacy_sources.exists() && !external_dir.exists() {
        std::fs::rename(&legacy_sources, &external_dir)?;
        if !flags.quiet {
            println!("Migrated sources/ → external/");
        }
    }

    // Migrate legacy registry.json to .equip/
    let legacy_registry = data.join("registry.json");
    let new_registry = crate::config::internal_dir().join("registry.json");
    if legacy_registry.exists() && !new_registry.exists() {
        std::fs::rename(&legacy_registry, &new_registry)?;
    }

    // Write .gitignore
    let gitignore_path = data.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "external/\n.equip/\n")?;
    }

    let default_config = crate::config::DEFAULT_CONFIG;
    std::fs::write(&path, default_config)?;
    if !flags.quiet {
        println!("Initialized equip at {}", data.display());
    }

    // If URL provided, fetch into cache and register as source
    if let Some(ref url_str) = url {
        let source_url = crate::source::SourceUrl::parse(url_str)?;
        let source_name = source_url.default_name();
        let cache_path = crate::config::cache_dir().join(&source_name);

        crate::source::fetch::fetch(&source_url, &cache_path, None)?;

        let parsed = crate::source::ParsedSource::parse(&cache_path)?
            .with_source_name(&source_name)
            .with_url(source_url.url_string());
        let registered = crate::source::normalize::normalize(&parsed)?;

        let data_dir = crate::config::data_dir();
        let mut registry = crate::registry::load_registry(&data_dir)?;
        registry.sources.push(registered);
        crate::registry::save_registry(&registry, &data_dir)?;

        let mut config = crate::config::load(flags.config_path())?;
        config.source.push(crate::config::SourceConfig {
            name: source_name.clone(),
            url: source_url.url_string(),
            source_type: source_url.source_type().to_string(),
            r#ref: None,
            mode: None,
        });
        crate::config::save(&config, flags.config_path())?;

        if !flags.quiet {
            println!("Added source '{}' from {}", source_name, url_str);
        }
    }

    // --- Interactive wizard steps ---

    // Step 1: git init the data dir
    let should_git_init = if data.join(".git").exists() {
        false
    } else if flags.quiet || !crate::prompt::is_interactive() {
        true
    } else {
        crate::prompt::confirm_or_override(
            "Initialize git in equip data dir? [Y/n]",
            "Y",
            flags.quiet,
        )
        .to_uppercase()
            != "N"
    };
    if should_git_init && !data.join(".git").exists() {
        let result = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&data)
            .output();
        match result {
            Ok(o) if o.status.success() => {
                if !flags.quiet {
                    println!("Initialized git in {}", data.display());
                }
            }
            Ok(o) => {
                if flags.verbose {
                    eprintln!(
                        "warning: git init failed: {}",
                        String::from_utf8_lossy(&o.stderr).trim()
                    );
                }
            }
            Err(_) => {
                if flags.verbose {
                    eprintln!("warning: git not found, skipping git init");
                }
            }
        }
    }

    // Step 2: detect and add agents
    let should_detect = if flags.quiet || !crate::prompt::is_interactive() {
        true
    } else {
        crate::prompt::confirm_or_override("Detect and add agents? [Y/n]", "Y", flags.quiet)
            .to_uppercase()
            != "N"
    };
    if should_detect {
        let mut config = crate::config::load(flags.config_path())?;
        let added = add_detected_agents(&mut config, flags.quiet);
        if added > 0 {
            crate::config::save(&config, flags.config_path())?;
        } else if !flags.quiet {
            println!("  No agents found");
        }
    }

    // Step 3: offer popular marketplaces (skip if URL was provided)
    if url.is_none() && crate::prompt::is_interactive() && !flags.quiet {
        let names: Vec<&str> = crate::marketplace::KNOWN_MARKETPLACES
            .iter()
            .map(|(name, _)| *name)
            .collect();
        let defaults: Vec<bool> = vec![true; names.len()];
        let selected = crate::prompt::multi_select(
            "Add popular skill sources?",
            &names,
            &defaults,
            flags.quiet,
        );

        if !selected.is_empty() {
            let mut config = crate::config::load(flags.config_path())?;
            let data_dir = crate::config::data_dir();
            let mut registry = crate::registry::load_registry(&data_dir)?;

            for idx in selected {
                let (name, url) = crate::marketplace::KNOWN_MARKETPLACES[idx];
                if config.source.iter().any(|s| s.url == url) {
                    continue;
                }
                let source_url = match crate::source::SourceUrl::parse(url) {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("warning: failed to parse '{}': {}", name, e);
                        continue;
                    }
                };
                let source_name = source_url.default_name();
                let cache_path = crate::config::cache_dir().join(&source_name);

                if !flags.quiet {
                    println!("Adding '{}'...", name);
                }
                match crate::source::fetch::fetch(&source_url, &cache_path, None) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("warning: failed to fetch '{}': {}", name, e);
                        continue;
                    }
                }
                match crate::source::ParsedSource::parse(&cache_path) {
                    Ok(parsed) => {
                        let parsed = parsed
                            .with_source_name(&source_name)
                            .with_url(url.to_string());
                        match crate::source::normalize::normalize(&parsed) {
                            Ok(registered) => {
                                registry.sources.retain(|s| s.name != source_name);
                                registry.sources.push(registered);
                                config.source.push(crate::config::SourceConfig {
                                    name: source_name.clone(),
                                    url: url.to_string(),
                                    source_type: "git".to_string(),
                                    r#ref: None,
                                    mode: None,
                                });
                                if !flags.quiet {
                                    println!("  Added source '{}'", source_name);
                                }
                            }
                            Err(e) => {
                                eprintln!("warning: failed to normalize '{}': {}", name, e)
                            }
                        }
                    }
                    Err(e) => eprintln!("warning: failed to parse '{}': {}", name, e),
                }
            }

            crate::registry::save_registry(&registry, &data_dir)?;
            crate::config::save(&config, flags.config_path())?;
        }
    }

    Ok(())
}
