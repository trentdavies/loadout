use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skittle",
    about = "Agent skill manager — add, update, and install skills across coding agents",
    version,
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Dry run — show what would change without making modifications
    #[arg(short = 'n', long = "dry-run", global = true)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Path to config file
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize skittle configuration
    Init {
        /// Optional source URL to populate cache (GitHub URL or local path)
        url: Option<String>,
    },

    /// Add a skill source
    Add {
        /// URL or path to the source
        url: String,

        /// Override the inferred source name
        #[arg(long)]
        source: Option<String>,

        /// Override the inferred plugin name
        #[arg(long)]
        plugin: Option<String>,

        /// Override the inferred skill name (single-skill sources only)
        #[arg(long)]
        skill: Option<String>,

        /// Deprecated: renamed to --source
        #[arg(long, hide = true)]
        name: Option<String>,

        /// Pin to a specific git ref (tag, branch, or commit SHA)
        #[arg(long, value_name = "REF")]
        r#ref: Option<String>,

        /// Symlink local directory sources instead of copying (default for local dirs)
        #[arg(long, conflicts_with = "copy")]
        symlink: bool,

        /// Copy local directory sources instead of symlinking
        #[arg(long, conflicts_with = "symlink")]
        copy: bool,
    },

    /// List skills, or show details for one
    List {
        /// Skill identity or glob pattern (plugin/skill, source:plugin/skill, or glob like "legal/*")
        patterns: Vec<String>,

        /// List external sources instead of skills
        #[arg(long)]
        external: bool,
    },

    /// Remove a skill source
    Remove {
        /// Source name (omit to select interactively)
        name: Option<String>,

        /// Force removal even if skills are installed
        #[arg(long)]
        force: bool,
    },

    /// Update source(s) from remote
    Update {
        /// Source name (omit to update all)
        name: Option<String>,

        /// Switch to a specific git ref (tag or branch). Use "latest" to unpin.
        #[arg(long, value_name = "REF")]
        r#ref: Option<String>,
    },

    /// Apply skills to targets
    Apply {
        /// Apply all configured skills
        #[arg(long)]
        all: bool,

        /// Apply a specific skill (plugin/skill)
        #[arg(long, value_name = "SKILL")]
        skill: Option<String>,

        /// Apply all skills from a plugin
        #[arg(long, value_name = "PLUGIN")]
        plugin: Option<String>,

        /// Apply a bundle of skills
        #[arg(long, value_name = "BUNDLE")]
        bundle: Option<String>,

        /// Target to apply to
        #[arg(long, value_name = "TARGET")]
        target: Option<String>,

        /// Force overwrite of changed skills without prompting
        #[arg(short, long)]
        force: bool,

        /// Interactively resolve conflicts for changed skills
        #[arg(short, long)]
        interactive: bool,
    },

    /// Uninstall skills from targets
    Uninstall {
        /// Uninstall a specific skill (plugin/skill)
        #[arg(long, value_name = "SKILL")]
        skill: Option<String>,

        /// Uninstall all skills from a plugin
        #[arg(long, value_name = "PLUGIN")]
        plugin: Option<String>,

        /// Uninstall a bundle of skills
        #[arg(long, value_name = "BUNDLE")]
        bundle: Option<String>,

        /// Target to uninstall from
        #[arg(long, value_name = "TARGET")]
        target: Option<String>,

        /// Actually perform the uninstall (default is dry run)
        #[arg(long)]
        force: bool,
    },

    /// Collect skills from a target back to source
    Collect {
        /// Skill name to collect
        #[arg(long, value_name = "SKILL")]
        skill: Option<String>,

        /// Target to collect from
        #[arg(long, value_name = "TARGET")]
        target: String,

        /// Adopt skill into plugins/ (make it yours)
        #[arg(long)]
        adopt: bool,

        /// Auto-adopt all untracked skills without prompting
        #[arg(long)]
        force: bool,
    },

    /// Show current status
    Status,

    /// Manage skill bundles
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Bundle {
        #[command(subcommand)]
        command: BundleCommand,
    },

    /// Manage install targets
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Target {
        #[command(subcommand)]
        command: TargetCommand,
    },

    /// Manage configuration
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

}

#[derive(Subcommand)]
pub enum BundleCommand {
    /// Create a new bundle
    Create {
        /// Bundle name
        name: String,
    },
    /// Delete a bundle
    Delete {
        /// Bundle name
        name: String,

        /// Force deletion of active bundle
        #[arg(long)]
        force: bool,
    },
    /// List all bundles, optionally filtered by name pattern
    List {
        /// Name patterns to filter by (glob supported)
        patterns: Vec<String>,
    },
    /// Show bundle details
    Show {
        /// Bundle name
        name: String,
    },
    /// Add skills to a bundle
    Add {
        /// Bundle name
        name: String,

        /// Skills to add (plugin/skill)
        #[arg(required = true)]
        skills: Vec<String>,
    },
    /// Remove skills from a bundle
    Drop {
        /// Bundle name
        name: String,

        /// Skills to remove (plugin/skill)
        #[arg(required = true)]
        skills: Vec<String>,
    },
    /// Activate a bundle (batch install its skills onto a target)
    Activate {
        /// Bundle name
        name: String,

        /// Target to activate on
        target: Option<String>,

        /// Activate on all configured targets
        #[arg(long)]
        all: bool,

        /// Actually perform the activation (default is dry run)
        #[arg(long)]
        force: bool,
    },
    /// Deactivate a bundle (batch uninstall its skills from a target)
    Deactivate {
        /// Bundle name
        name: String,

        /// Target to deactivate from
        target: Option<String>,

        /// Deactivate from all configured targets
        #[arg(long)]
        all: bool,

        /// Actually perform the deactivation (default is dry run)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum TargetCommand {
    /// Add an install target
    Add {
        /// Agent type (claude, codex, cursor, etc.)
        agent: String,

        /// Path to target directory
        path: Option<String>,

        /// Name for this target
        #[arg(long)]
        name: Option<String>,

        /// Scope: machine or repo
        #[arg(long, default_value = "machine")]
        scope: String,

        /// Sync mode: auto or explicit
        #[arg(long, default_value = "auto")]
        sync: String,
    },
    /// Remove a target
    Remove {
        /// Target name
        name: String,

        /// Actually perform the removal (default is dry run)
        #[arg(long)]
        force: bool,
    },
    /// List all targets
    List,
    /// Show target details
    Show {
        /// Target name
        name: String,
    },
    /// Detect agent installations and prompt to add them
    Detect {
        /// Automatically add all detected targets without prompting
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Open config in editor
    Edit,
}

/// Extract the domain from a URL. Returns empty string for local paths.
fn extract_domain(url: &str) -> String {
    // git@github.com:org/repo.git → github.com
    if let Some(rest) = url.strip_prefix("git@") {
        return rest.split(':').next().unwrap_or("").to_string();
    }
    // https://github.com/... → github.com
    if let Some(after_scheme) = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git://"))
        .or_else(|| url.strip_prefix("ssh://"))
    {
        let host = after_scheme.split('/').next().unwrap_or("");
        // strip user@ prefix (ssh://git@github.com/...)
        return host.split('@').last().unwrap_or(host).to_string();
    }
    String::new()
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init { url } => {
            let path = crate::config::config_path(cli.config.as_deref());
            if path.exists() {
                if url.is_some() && !cli.quiet {
                    println!("Config already exists at {}. Use `skittle add` instead.", path.display());
                } else if !cli.quiet {
                    println!("Config already exists at {}. Use `skittle config edit` to modify.", path.display());
                }
                return Ok(());
            }
            // Create directory structure
            let data = crate::config::data_dir();
            std::fs::create_dir_all(&data)?;
            std::fs::create_dir_all(crate::config::plugins_dir())?;
            std::fs::create_dir_all(crate::config::cache_dir())?;
            std::fs::create_dir_all(crate::config::internal_dir())?;

            // Legacy migration: rename sources/ to external/
            let legacy_sources = data.join("sources");
            let external_dir = data.join("external");
            if legacy_sources.exists() && !external_dir.exists() {
                std::fs::rename(&legacy_sources, &external_dir)?;
                if !cli.quiet {
                    println!("Migrated sources/ → external/");
                }
            }

            // Migrate legacy registry.json to .skittle/
            let legacy_registry = data.join("registry.json");
            let new_registry = crate::config::internal_dir().join("registry.json");
            if legacy_registry.exists() && !new_registry.exists() {
                std::fs::rename(&legacy_registry, &new_registry)?;
            }

            // Write .gitignore
            let gitignore_path = data.join(".gitignore");
            if !gitignore_path.exists() {
                std::fs::write(&gitignore_path, "external/\n.skittle/\n")?;
            }

            let default_config = crate::config::DEFAULT_CONFIG;
            std::fs::write(&path, default_config)?;
            if !cli.quiet {
                println!("Initialized skittle at {}", data.display());
            }

            // If URL provided, fetch into cache and register as source
            if let Some(ref url_str) = url {
                let source_url = crate::source::SourceUrl::parse(url_str)?;
                let source_name = source_url.default_name();
                let cache_path = crate::config::cache_dir().join(&source_name);

                crate::source::fetch::fetch(&source_url, &cache_path, None)?;

                let structure = crate::source::detect::detect(&cache_path)?;
                let registered = crate::source::normalize::normalize(
                    &source_name, &cache_path, &structure,
                )?;

                let data_dir = crate::config::data_dir();
                let mut registry = crate::registry::load_registry(&data_dir)?;
                registry.sources.push(registered);
                crate::registry::save_registry(&registry, &data_dir)?;

                let mut config = crate::config::load(cli.config.as_deref())?;
                config.source.push(crate::config::SourceConfig {
                    name: source_name.clone(),
                    url: source_url.url_string(),
                    source_type: source_url.source_type().to_string(),
                    r#ref: None,
                    mode: None,
                });
                crate::config::save(&config, cli.config.as_deref())?;

                if !cli.quiet {
                    println!("Added source '{}' from {}", source_name, url_str);
                }
            }

            Ok(())
        }
        Command::Add { url, source, plugin, skill, name, r#ref, symlink, copy } => {
            // Backward-compat: error on deprecated --name
            if name.is_some() {
                anyhow::bail!("`--name` has been renamed to `--source`");
            }

            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();

            let source_url = crate::source::SourceUrl::parse(&url)?;
            let default_source = source_url.default_name();
            let source_name = if let Some(s) = source {
                s
            } else {
                crate::prompt::confirm_or_override("Source name", &default_source, cli.quiet)
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
                        crate::prompt::prompt_fetch_mode(cli.quiet) == "symlink"
                    }
                }
                _ => false, // non-local or single-file: always copy
            };

            if !cli.dry_run {
                crate::source::fetch::fetch_with_mode(&source_url, &cache_path, r#ref.as_deref(), use_symlink)?;

                let structure = crate::source::detect::detect(&cache_path)?;

                // Determine default plugin/skill names from structure for prompting
                let overrides = {
                    use crate::source::detect::SourceStructure;

                    let plugin_override: Option<String> = if plugin.is_some() {
                        plugin
                    } else {
                        // Prompt only when the inferred plugin differs from source
                        let default_plugin = match &structure {
                            SourceStructure::SingleFile { .. } | SourceStructure::SingleSkillDir { .. } => {
                                // plugin = source_name, no point prompting
                                None
                            }
                            SourceStructure::FlatSkills => {
                                let dir = cache_path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&source_name);
                                if dir == source_name { None } else { Some(dir.to_string()) }
                            }
                            SourceStructure::SinglePlugin => {
                                let plugin_json = cache_path.join(".claude-plugin/plugin.json");
                                if plugin_json.exists() {
                                    let m = crate::source::manifest::load_plugin_manifest(&plugin_json)?;
                                    if m.name == source_name { None } else { Some(m.name) }
                                } else {
                                    let n = cache_path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unnamed")
                                        .to_string();
                                    if n == source_name { None } else { Some(n) }
                                }
                            }
                            SourceStructure::Marketplace => None,
                        };
                        if let Some(ref dp) = default_plugin {
                            let confirmed = crate::prompt::confirm_or_override("Plugin name", dp, cli.quiet);
                            // Leak the confirmed string into the overrides
                            if confirmed != *dp {
                                Some(confirmed)
                            } else {
                                None // use the natural inference
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
                                let confirmed = crate::prompt::confirm_or_override("Skill name", skill_name, cli.quiet);
                                if confirmed != *skill_name { Some(confirmed) } else { None }
                            }
                            SourceStructure::SingleSkillDir { skill_name } => {
                                let confirmed = crate::prompt::confirm_or_override("Skill name", skill_name, cli.quiet);
                                if confirmed != *skill_name { Some(confirmed) } else { None }
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

                let registered = crate::source::normalize::normalize_with(
                    &source_name, &cache_path, &structure, &norm_overrides,
                )?;

                // In non-interactive/quiet mode, show what was resolved
                if !cli.quiet && !crate::prompt::is_interactive() {
                    for p in &registered.plugins {
                        for s in &p.skills {
                            eprintln!("resolved: {}", crate::output::plain_identity(&source_name, &p.name, &s.name));
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
                    mode: if use_symlink { Some("symlink".to_string()) } else { None },
                });
                crate::config::save(&config, config_path_str)?;
            }

            if !cli.quiet {
                println!("Added source '{}'", source_name);
            }
            Ok(())
        }
        Command::List { patterns, external } => {
            let data_dir = crate::config::data_dir();
            let config_for_list = crate::config::load(cli.config.as_deref())?;
            let registry = crate::registry::load_registry(&data_dir)?;

            if external {
                // List external sources in table format
                if cli.json {
                    let entries: Vec<serde_json::Value> = config_for_list.source.iter().map(|src| {
                        let skill_count: usize = registry.sources.iter()
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
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&entries)?);
                    return Ok(());
                }

                if config_for_list.source.is_empty() {
                    let output = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    output.info("No sources configured. Use `skittle add` to add one.");
                    return Ok(());
                }

                let rows: Vec<Vec<String>> = config_for_list.source.iter().map(|src| {
                    let skill_count: usize = registry.sources.iter()
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
                }).collect();

                let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                out.table(&["NAME", "TYPE", "DOMAIN", "REF", "SKILLS", "MODE"], &rows);
                return Ok(());
            }

            if patterns.len() == 1 && !crate::registry::is_glob(&patterns[0]) {
                // Show details for a single exact skill
                let identity = &patterns[0];
                let (source_name, plugin_name, skill) = registry.find_skill(identity)?;

                if cli.json {
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

                let out = crate::output::Output::from_flags(
                    cli.json, cli.quiet, cli.verbose,
                );
                out.status("Identity", &crate::output::format_identity(source_name, plugin_name, &skill.name));
                out.status("Description", skill.description.as_deref().unwrap_or("(none)"));
                if cli.verbose {
                    out.status("Path", &skill.path.display().to_string());
                }
            } else {
                // List skills: all (no patterns), or filtered by glob/exact patterns
                let skills: Vec<(&str, &crate::registry::RegisteredPlugin, &crate::registry::RegisteredSkill)> = if patterns.is_empty() {
                    registry.all_skills()
                } else {
                    let mut seen = std::collections::HashSet::new();
                    let mut result = Vec::new();
                    for pat in &patterns {
                        if crate::registry::is_glob(pat) {
                            for triple in registry.match_skills(pat) {
                                let id = crate::output::plain_identity(triple.0, &triple.1.name, &triple.2.name);
                                if seen.insert(id) {
                                    result.push(triple);
                                }
                            }
                        } else {
                            match registry.find_skill(pat) {
                                Ok((src, plug, sk)) => {
                                    let id = crate::output::plain_identity(src, plug, &sk.name);
                                    if seen.insert(id) {
                                        result.push((src, registry.sources.iter()
                                            .flat_map(|s| s.plugins.iter())
                                            .find(|p| p.name == plug).unwrap(), sk));
                                    }
                                }
                                Err(e) => anyhow::bail!("{}", e),
                            }
                        }
                    }
                    result
                };

                if cli.json {
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
                    let output = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    if skills.is_empty() {
                        if patterns.is_empty() {
                            output.info("No skills found. Add a source with `skittle add`");
                        } else {
                            output.info("No skills matched the given pattern(s)");
                        }
                    } else {
                        for (source_name, plugin, skill) in &skills {
                            println!("{}", crate::output::format_identity(source_name, &plugin.name, &skill.name));
                        }
                    }
                }
            }
            Ok(())
        }
        Command::Apply { all, skill, plugin, bundle, target, force, interactive } => {
            if !all && skill.is_none() && plugin.is_none() && bundle.is_none() {
                eprintln!("error: apply requires --all, --skill, --plugin, or --bundle");
                std::process::exit(2);
            }

            let config_path_str = cli.config.as_deref();
            let config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            // Determine which targets to apply to
            let targets: Vec<&crate::config::TargetConfig> = if let Some(ref t) = target {
                let tc = config.target.iter()
                    .find(|tc| tc.name == *t)
                    .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                vec![tc]
            } else {
                config.target.iter().filter(|t| t.sync == "auto").collect()
            };

            if targets.is_empty() {
                anyhow::bail!("no targets configured. Use `skittle target add` first.");
            }

            // Collect skills to apply with provenance: (source, plugin, skill)
            let mut skills_to_apply: Vec<(&str, &str, &crate::registry::RegisteredSkill)> = Vec::new();

            if all {
                for src in &registry.sources {
                    for p in &src.plugins {
                        for s in &p.skills {
                            skills_to_apply.push((&src.name, &p.name, s));
                        }
                    }
                }
            }

            if let Some(ref skill_id) = skill {
                let (src, plug, s) = registry.find_skill(skill_id)?;
                skills_to_apply.push((src, plug, s));
            }

            if let Some(ref plugin_name) = plugin {
                let (src_name, p) = registry.find_plugin(plugin_name)
                    .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_name))?;
                for s in &p.skills {
                    skills_to_apply.push((src_name, &p.name, s));
                }
            }

            if let Some(ref bundle_name) = bundle {
                let bundle_cfg = config.bundle.get(bundle_name)
                    .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", bundle_name))?;
                for skill_id in &bundle_cfg.skills {
                    let (src, plug, s) = registry.find_skill(skill_id)?;
                    skills_to_apply.push((src, plug, s));
                }
            }

            // Apply to each target with conflict detection
            let mut new_count: usize = 0;
            let mut updated_count: usize = 0;
            let mut unchanged_count: usize = 0;
            let mut conflict_skipped: usize = 0;
            let mut force_remaining = force;
            let mut reg = registry.clone();

            for tc in &targets {
                let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;

                // First pass: detect conflicts in default mode (no --force, no -i)
                if !force && !interactive && !cli.dry_run {
                    let mut conflicts = Vec::new();
                    for (_, _, s) in &skills_to_apply {
                        let status = adapter.compare_skill(s, &tc.path)?;
                        if status == crate::target::SkillStatus::Changed {
                            conflicts.push(s.name.clone());
                        }
                    }
                    if !conflicts.is_empty() {
                        eprintln!("error: {} skill(s) have changed at target '{}':", conflicts.len(), tc.name);
                        for name in &conflicts {
                            eprintln!("  - {}", name);
                        }
                        eprintln!();
                        eprintln!("Use --force to overwrite, or -i for interactive resolution.");
                        std::process::exit(1);
                    }
                }

                for (src_name, plug_name, s) in &skills_to_apply {
                    let status = adapter.compare_skill(s, &tc.path)?;

                    if cli.dry_run {
                        if !cli.quiet {
                            let label = match status {
                                crate::target::SkillStatus::New => "new",
                                crate::target::SkillStatus::Unchanged => "unchanged",
                                crate::target::SkillStatus::Changed => "changed",
                            };
                            println!("  (dry run) {} → {} [{}]", crate::output::format_identity(src_name, plug_name, &s.name), tc.name, label);
                        }
                        continue;
                    }

                    match status {
                        crate::target::SkillStatus::Unchanged => {
                            unchanged_count += 1;
                            continue;
                        }
                        crate::target::SkillStatus::New => {
                            adapter.install_skill(s, &tc.path)?;
                            record_provenance(&mut reg, &data_dir, tc, src_name, plug_name, s);
                            new_count += 1;
                        }
                        crate::target::SkillStatus::Changed => {
                            if force_remaining {
                                adapter.install_skill(s, &tc.path)?;
                                record_provenance(&mut reg, &data_dir, tc, src_name, plug_name, s);
                                updated_count += 1;
                            } else if interactive {
                                let action = prompt_conflict(s, &adapter, &tc.path)?;
                                match action {
                                    ConflictAction::Skip => {
                                        conflict_skipped += 1;
                                    }
                                    ConflictAction::Overwrite => {
                                        adapter.install_skill(s, &tc.path)?;
                                        record_provenance(&mut reg, &data_dir, tc, src_name, plug_name, s);
                                        updated_count += 1;
                                    }
                                    ConflictAction::ForceAll => {
                                        adapter.install_skill(s, &tc.path)?;
                                        record_provenance(&mut reg, &data_dir, tc, src_name, plug_name, s);
                                        updated_count += 1;
                                        force_remaining = true;
                                    }
                                    ConflictAction::Quit => {
                                        // Save what we have so far and exit
                                        crate::registry::save_registry(&reg, &data_dir)?;
                                        print_apply_summary(new_count, updated_count, unchanged_count, conflict_skipped, cli.quiet);
                                        return Ok(());
                                    }
                                }
                            }
                            // Default mode with conflicts is handled above (exits before this loop)
                        }
                    }
                }
            }

            if !cli.dry_run {
                crate::registry::save_registry(&reg, &data_dir)?;
            }

            if !cli.quiet && !cli.dry_run {
                print_apply_summary(new_count, updated_count, unchanged_count, conflict_skipped, cli.quiet);
            }
            Ok(())
        }
        Command::Uninstall { skill, plugin, bundle, target, force } => {
            if skill.is_none() && plugin.is_none() && bundle.is_none() {
                eprintln!("error: uninstall requires --skill, --plugin, or --bundle");
                std::process::exit(2);
            }

            let config_path_str = cli.config.as_deref();
            let config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            // Determine targets
            let targets: Vec<&crate::config::TargetConfig> = if let Some(ref t) = target {
                let tc = config.target.iter()
                    .find(|tc| tc.name == *t)
                    .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                vec![tc]
            } else {
                config.target.iter().filter(|t| t.sync == "auto").collect()
            };

            // Collect skill names to uninstall
            let mut skill_names: Vec<String> = Vec::new();

            if let Some(ref skill_id) = skill {
                let (_, _, s) = registry.find_skill(skill_id)?;
                skill_names.push(s.name.clone());
            }

            if let Some(ref plugin_name) = plugin {
                let (_, p) = registry.find_plugin(plugin_name)
                    .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_name))?;
                for s in &p.skills {
                    skill_names.push(s.name.clone());
                }
            }

            if let Some(ref bundle_name) = bundle {
                let bundle_cfg = config.bundle.get(bundle_name)
                    .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", bundle_name))?;
                for skill_id in &bundle_cfg.skills {
                    let (_, _, s) = registry.find_skill(skill_id)?;
                    skill_names.push(s.name.clone());
                }
            }

            let execute = force && !cli.dry_run;
            for tc in &targets {
                let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
                for name in &skill_names {
                    if execute {
                        adapter.uninstall_skill(name, &tc.path)?;
                    } else if !cli.quiet {
                        // Look up provenance for colored identity
                        let identity = registry.installed.get(&tc.name)
                            .and_then(|m| m.get(name))
                            .map(|info| crate::output::format_identity(&info.source, &info.plugin, &info.skill))
                            .unwrap_or_else(|| name.clone());
                        println!("  would uninstall {} from {}", identity, tc.name);
                    }
                }
            }

            if !execute && !cli.quiet {
                println!("Use --force to uninstall");
            } else if !cli.quiet {
                println!("Uninstalled {} skill(s) from {} target(s)", skill_names.len(), targets.len());
            }
            Ok(())
        }
        Command::Collect { skill, target, adopt, force } => {
            let config = crate::config::load(cli.config.as_deref())?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;
            let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);

            let tc = config.target.iter()
                .find(|t| t.name == target)
                .ok_or_else(|| anyhow::anyhow!("target '{}' not found", target))?;
            let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
            let installed_on_target = adapter.installed_skills(&tc.path)?;

            if let Some(ref skill_name) = skill {
                // Collect a specific skill
                let target_skill_dir = tc.path.join("skills").join(skill_name);
                if !target_skill_dir.exists() {
                    anyhow::bail!("skill '{}' not found on target '{}'", skill_name, target);
                }

                if adopt {
                    // Copy to plugins/
                    let provenance = registry.installed.get(&target)
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
                    copy_dir_all(&target_skill_dir, &dest_skill)?;

                    // Create plugin.json if missing
                    let plugin_json_dir = dest_plugin.join(".claude-plugin");
                    let plugin_json = plugin_json_dir.join("plugin.json");
                    if !plugin_json.exists() {
                        std::fs::create_dir_all(&plugin_json_dir)?;
                        let json = serde_json::json!({"name": plugin_name});
                        std::fs::write(&plugin_json, serde_json::to_string_pretty(&json)?)?;
                    }

                    // Regenerate marketplace
                    generate_marketplace(&data_dir)?;
                    let identity = crate::output::format_identity(&source_name, &plugin_name, skill_name);
                    out.success(&format!("Adopted {}", identity));
                } else {
                    // Copy back to origin
                    let provenance = registry.installed.get(&target)
                        .and_then(|m| m.get(skill_name));

                    if let Some(info) = provenance {
                        let dest = data_dir.join(&info.origin);
                        std::fs::create_dir_all(&dest)?;
                        copy_dir_all(&target_skill_dir, &dest)?;
                        let identity = crate::output::format_identity(&info.source, &info.plugin, &info.skill);
                        out.success(&format!("Collected {} → {}", identity, info.origin));
                    } else {
                        out.warn(&format!("'{}' has no provenance. Use --adopt to claim it.", skill_name));
                    }
                }
            } else {
                // Scan target for all skills
                let target_installs = registry.installed.get(&target).cloned().unwrap_or_default();

                let mut tracked = Vec::new();
                let mut untracked = Vec::new();

                for skill_name in &installed_on_target {
                    if let Some(info) = target_installs.get(skill_name) {
                        tracked.push((skill_name.clone(), info.clone()));
                    } else {
                        untracked.push(skill_name.clone());
                    }
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

                    let should_adopt = if force {
                        true
                    } else if !untracked.is_empty() {
                        eprint!("Adopt {} untracked skill(s) into plugins/local? [y/N] ", untracked.len());
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap_or(0);
                        input.trim().eq_ignore_ascii_case("y")
                    } else {
                        false
                    };

                    if should_adopt {
                        let local_plugin = crate::config::plugins_dir().join("local");
                        for name in &untracked {
                            let target_skill_dir = tc.path.join("skills").join(name);
                            let dest = local_plugin.join("skills").join(name);
                            std::fs::create_dir_all(&dest)?;
                            copy_dir_all(&target_skill_dir, &dest)?;
                            let identity = crate::output::format_identity("local", "local", name);
                            out.success(&format!("Adopted {}", identity));
                        }

                        // Create plugin.json for local plugin if missing
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
                    out.info("No skills found on target.");
                }
            }

            crate::registry::save_registry(&registry, &data_dir)?;
            Ok(())
        }
        Command::Status => {
            let config = crate::config::load(cli.config.as_deref())?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            // Count installed skills across targets
            let mut total_installed = 0;
            for tc in &config.target {
                let adapter = crate::target::resolve_adapter(tc, &config.adapter).ok();
                if let Some(a) = adapter {
                    if let Ok(skills) = a.installed_skills(&tc.path) {
                        total_installed += skills.len();
                    }
                }
            }

            let total_skills: usize = registry.sources.iter()
                .flat_map(|s| &s.plugins)
                .map(|p| p.skills.len())
                .sum();

            if cli.json {
                let json = serde_json::json!({
                    "sources": config.source.len(),
                    "targets": config.target.len(),
                    "plugins": registry.sources.iter().flat_map(|s| &s.plugins).count(),
                    "skills": total_skills,
                    "installed": total_installed,
                    "bundles": config.bundle.len(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(
                cli.json, cli.quiet, cli.verbose,
            );
            out.status("Sources", &config.source.len().to_string());
            out.status("Targets", &config.target.len().to_string());
            out.status("Plugins", &registry.sources.iter().flat_map(|s| &s.plugins).count().to_string());
            out.status("Skills", &total_skills.to_string());
            out.status("Installed", &total_installed.to_string());
            out.status("Bundles", &config.bundle.len().to_string());

            // Show source refs if any are pinned/tracking
            let sources_with_refs: Vec<_> = config.source.iter()
                .filter(|s| s.r#ref.is_some())
                .collect();
            if !sources_with_refs.is_empty() {
                out.info("");
                out.info("Source versions:");
                for src in &config.source {
                    let version = src.r#ref.as_deref().unwrap_or("latest");
                    out.info(&format!("  {} @ {}", src.name, version));
                }
            }

            Ok(())
        }
        Command::Remove { name, force } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();

            let name = match name {
                Some(n) => n,
                None => {
                    let source_names: Vec<String> = config.source.iter()
                        .map(|s| s.name.clone())
                        .collect();
                    if source_names.is_empty() {
                        anyhow::bail!("no sources configured");
                    }
                    crate::prompt::select_from("Select source to remove", &source_names, cli.quiet)?
                }
            };

            if !config.source.iter().any(|s| s.name == name) {
                anyhow::bail!("source '{}' not found", name);
            }

            // Check if any skills from this source are installed on targets
            let registry = crate::registry::load_registry(&data_dir)?;
            let mut installed_on: Vec<String> = Vec::new();
            if let Some(reg_src) = registry.sources.iter().find(|s| s.name == name) {
                let skill_names: Vec<&str> = reg_src.plugins.iter()
                    .flat_map(|p| p.skills.iter().map(|s| s.name.as_str()))
                    .collect();
                for tc in &config.target {
                    let target_path = std::path::PathBuf::from(&tc.path);
                    if let Ok(adapter) = crate::target::resolve_adapter(tc, &config.adapter) {
                        if let Ok(installed) = adapter.installed_skills(&target_path) {
                            for sk in &skill_names {
                                if installed.contains(&sk.to_string()) {
                                    installed_on.push(tc.name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if !installed_on.is_empty() && !cli.quiet {
                eprintln!("warning: source '{}' has installed skills on: {}", name, installed_on.join(", "));
            }

            let execute = force && !cli.dry_run;
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

            if !cli.quiet {
                if execute {
                    println!("Removed source '{}'", name);
                } else {
                    println!("Would remove source '{}'", name);
                    println!("Use --force to remove");
                }
            }
            Ok(())
        }
        Command::Update { name, r#ref: update_ref } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            if update_ref.is_some() && name.is_none() {
                anyhow::bail!("--ref requires a source name (e.g., skittle update my-source --ref v2.0)");
            }

            // Determine which sources to update
            let sources_to_update: Vec<&crate::config::SourceConfig> = if let Some(ref n) = name {
                let src = config.source.iter()
                    .find(|s| s.name == *n)
                    .ok_or_else(|| anyhow::anyhow!("source '{}' not found", n))?;
                vec![src]
            } else {
                if config.source.is_empty() {
                    if !cli.quiet {
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
                if !cli.quiet {
                    println!("Updating '{}'...", src.name);
                }

                if cli.dry_run {
                    if !cli.quiet {
                        println!("  (dry run) would re-fetch from {}", src.url);
                    }
                    updated_count += 1;
                    continue;
                }

                let cache_path = crate::config::cache_dir().join(&src.name);

                // Re-fetch based on source type
                let source_url = match crate::source::SourceUrl::parse(&src.url) {
                    Ok(u) => u,
                    Err(e) => {
                        errors.push(format!("{}: {}", src.name, e));
                        continue;
                    }
                };

                // For symlinked local sources, skip re-fetch — just re-detect
                // For copied local sources, remove cache and re-copy
                // For git sources, update with ref awareness
                let is_symlinked = src.mode.as_deref() == Some("symlink");
                match &source_url {
                    crate::source::SourceUrl::Local(path) => {
                        if is_symlinked {
                            if !cli.quiet {
                                println!("  (symlinked, re-detecting)");
                            }
                            // Symlink points to live source — skip re-fetch
                        } else {
                            if cache_path.exists() {
                                std::fs::remove_dir_all(&cache_path)?;
                            }
                            if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path, None) {
                                errors.push(format!("{}: {}", src.name, e));
                                continue;
                            }
                        }
                        let _ = path; // used via source_url
                    }
                    crate::source::SourceUrl::Git(_) => {
                        if let Some(ref new_ref) = update_ref {
                            // Ref switch: fetch + checkout new ref + update config
                            if !cache_path.exists() {
                                let effective_ref = if new_ref == "latest" { None } else { Some(new_ref.as_str()) };
                                if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path, effective_ref) {
                                    errors.push(format!("{}: {}", src.name, e));
                                    continue;
                                }
                            } else if new_ref == "latest" {
                                // Unpin: fetch + reset to default branch
                                if let Err(e) = crate::source::fetch::update_git(&cache_path, None) {
                                    errors.push(format!("{}: {}", src.name, e));
                                    continue;
                                }
                            } else {
                                if let Err(e) = crate::source::fetch::switch_ref(&cache_path, new_ref) {
                                    errors.push(format!("{}: {}", src.name, e));
                                    continue;
                                }
                            }
                            ref_changed = true;
                        } else if cache_path.exists() {
                            match crate::source::fetch::update_git_ref(&cache_path, src.r#ref.as_deref()) {
                                Ok(None) => {
                                    // Pinned to a tag — warn and skip
                                    if !cli.quiet {
                                        let tag = src.r#ref.as_deref().unwrap_or("unknown");
                                        eprintln!("warning: source '{}' is pinned to {}, skipping", src.name, tag);
                                    }
                                    continue;
                                }
                                Ok(Some(_)) => {}
                                Err(e) => {
                                    errors.push(format!("{}: {}", src.name, e));
                                    continue;
                                }
                            }
                        } else {
                            if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path, src.r#ref.as_deref()) {
                                errors.push(format!("{}: {}", src.name, e));
                                continue;
                            }
                        }
                    }
                    crate::source::SourceUrl::Archive(_) => {
                        // Re-extract archive to cache
                        if cache_path.exists() {
                            std::fs::remove_dir_all(&cache_path)?;
                        }
                        if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path, None) {
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
                    Ok(registered) => {
                        updated_registry.sources.retain(|s| s.name != src.name);
                        updated_registry.sources.push(registered);
                        updated_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("{}: normalization failed: {}", src.name, e));
                    }
                }
            }

            if !cli.dry_run {
                crate::registry::save_registry(&updated_registry, &data_dir)?;
                if ref_changed {
                    if let Some(ref new_ref) = update_ref {
                        if let Some(ref source_name) = name {
                            if let Some(cfg_src) = config.source.iter_mut().find(|s| s.name == *source_name) {
                                cfg_src.r#ref = if new_ref == "latest" { None } else { Some(new_ref.clone()) };
                            }
                        }
                    }
                    crate::config::save(&config, config_path_str)?;
                }
            }

            if !cli.quiet {
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
        Command::Bundle { command: bundle_cmd } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();

            match bundle_cmd {
                BundleCommand::Create { name } => {
                    if config.bundle.contains_key(&name) {
                        anyhow::bail!("bundle '{}' already exists", name);
                    }
                    config.bundle.insert(name.clone(), crate::config::BundleConfig::default());
                    crate::config::save(&config, config_path_str)?;
                    if !cli.quiet {
                        println!("Created bundle '{}'", name);
                    }
                    Ok(())
                }
                BundleCommand::Delete { name, force } => {
                    if !config.bundle.contains_key(&name) {
                        anyhow::bail!("bundle '{}' not found", name);
                    }

                    let execute = force && !cli.dry_run;
                    if execute {
                        config.bundle.remove(&name);
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        if execute {
                            println!("Deleted bundle '{}'", name);
                        } else {
                            println!("Would delete bundle '{}'", name);
                            println!("Use --force to delete");
                        }
                    }
                    Ok(())
                }
                BundleCommand::List { patterns } => {
                    let bundles: Vec<(&String, &crate::config::BundleConfig)> = if patterns.is_empty() {
                        config.bundle.iter().collect()
                    } else {
                        config.bundle.iter().filter(|(name, _)| {
                            patterns.iter().any(|pat| {
                                if crate::registry::is_glob(pat) {
                                    glob_match::glob_match(pat, name)
                                } else {
                                    *name == pat
                                }
                            })
                        }).collect()
                    };

                    if cli.json {
                        let entries: Vec<serde_json::Value> = bundles.iter().map(|(name, b)| {
                            serde_json::json!({
                                "name": name,
                                "skills": b.skills.len(),
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if bundles.is_empty() {
                        if !cli.quiet {
                            if patterns.is_empty() {
                                println!("No bundles configured. Use `skittle bundle create` to create one.");
                            } else {
                                println!("No bundles matched the given pattern(s)");
                            }
                        }
                        return Ok(());
                    }

                    let rows: Vec<Vec<String>> = bundles.iter().map(|(name, b)| {
                        vec![
                            (*name).clone(),
                            b.skills.len().to_string(),
                        ]
                    }).collect();

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose,
                    );
                    out.table(&["BUNDLE", "SKILLS"], &rows);
                    Ok(())
                }
                BundleCommand::Show { name } => {
                    let bundle = config.bundle.get(&name)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", name))?;

                    if cli.json {
                        let json = serde_json::json!({
                            "name": name,
                            "skills": bundle.skills,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose,
                    );
                    out.status("Bundle", &name);
                    out.status("Skills", &bundle.skills.len().to_string());

                    if !bundle.skills.is_empty() {
                        out.info("");
                        let tree: Vec<(usize, String)> = bundle.skills.iter()
                            .map(|s| (0, s.clone()))
                            .collect();
                        out.tree(&tree);
                    }

                    Ok(())
                }
                BundleCommand::Add { name, skills } => {
                    let bundle = config.bundle.get_mut(&name)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", name))?;

                    let registry = crate::registry::load_registry(&data_dir)?;

                    let mut added = 0usize;
                    for skill_id in &skills {
                        if crate::registry::is_glob(skill_id) {
                            let matches = registry.match_skills(skill_id);
                            if matches.is_empty() {
                                anyhow::bail!("no skills matched pattern '{}'", skill_id);
                            }
                            for (src, plugin, skill) in &matches {
                                let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
                                if !bundle.skills.contains(&fq) {
                                    bundle.skills.push(fq);
                                    added += 1;
                                }
                            }
                        } else {
                            // Validate skill exists and resolve to fully qualified
                            let (src, plug, sk) = registry.find_skill(skill_id)?;
                            let fq = crate::output::plain_identity(src, plug, &sk.name);
                            if !bundle.skills.contains(&fq) {
                                bundle.skills.push(fq);
                                added += 1;
                            }
                        }
                    }

                    crate::config::save(&config, config_path_str)?;
                    if !cli.quiet {
                        println!("Added {} skill(s) to bundle '{}'", added, name);
                    }
                    Ok(())
                }
                BundleCommand::Drop { name, skills } => {
                    let bundle = config.bundle.get_mut(&name)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", name))?;

                    for skill_id in &skills {
                        bundle.skills.retain(|s| s != skill_id);
                    }

                    crate::config::save(&config, config_path_str)?;
                    if !cli.quiet {
                        println!("Dropped {} skill(s) from bundle '{}'", skills.len(), name);
                    }
                    Ok(())
                }
                BundleCommand::Activate { name, target, all, force } => {
                    if target.is_none() && !all {
                        anyhow::bail!("provide a <target> or use --all");
                    }

                    let bundle = config.bundle.get(&name)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", name))?
                        .clone();
                    let registry = crate::registry::load_registry(&data_dir)?;

                    let targets: Vec<&crate::config::TargetConfig> = if all {
                        config.target.iter().collect()
                    } else {
                        let t = target.as_ref().unwrap();
                        let tc = config.target.iter()
                            .find(|tc| tc.name == *t)
                            .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                        vec![tc]
                    };

                    let execute = force && !cli.dry_run;
                    let mut installed_count = 0usize;
                    for tc in &targets {
                        let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
                        let already = adapter.installed_skills(&tc.path).unwrap_or_default();

                        for skill_id in &bundle.skills {
                            let (_, _, s) = registry.find_skill(skill_id)?;
                            if already.contains(&s.name) {
                                continue; // idempotent: skip silently
                            }
                            if execute {
                                adapter.install_skill(s, &tc.path)?;
                                installed_count += 1;
                            } else if !cli.quiet {
                                println!("  would install {} on {}", skill_id, tc.name);
                            }
                        }
                    }

                    if !cli.quiet {
                        if execute {
                            println!("Activated bundle '{}' ({} skill(s) installed)", name, installed_count);
                        } else {
                            println!("Use --force to activate");
                        }
                    }
                    Ok(())
                }
                BundleCommand::Deactivate { name, target, all, force } => {
                    if target.is_none() && !all {
                        anyhow::bail!("provide a <target> or use --all");
                    }

                    let bundle = config.bundle.get(&name)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", name))?
                        .clone();
                    let registry = crate::registry::load_registry(&data_dir)?;

                    let targets: Vec<&crate::config::TargetConfig> = if all {
                        config.target.iter().collect()
                    } else {
                        let t = target.as_ref().unwrap();
                        let tc = config.target.iter()
                            .find(|tc| tc.name == *t)
                            .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                        vec![tc]
                    };

                    let execute = force && !cli.dry_run;
                    let mut removed_count = 0usize;
                    for tc in &targets {
                        let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
                        let already = adapter.installed_skills(&tc.path).unwrap_or_default();

                        for skill_id in &bundle.skills {
                            let (_, _, s) = registry.find_skill(skill_id)?;
                            if !already.contains(&s.name) {
                                continue; // idempotent: skip silently
                            }
                            if execute {
                                adapter.uninstall_skill(&s.name, &tc.path)?;
                                removed_count += 1;
                            } else if !cli.quiet {
                                println!("  would uninstall {} from {}", skill_id, tc.name);
                            }
                        }
                    }

                    if !cli.quiet {
                        if execute {
                            println!("Deactivated bundle '{}' ({} skill(s) removed)", name, removed_count);
                        } else {
                            println!("Use --force to deactivate");
                        }
                    }
                    Ok(())
                }
            }
        }
        Command::Target { command: target_cmd } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;

            // Known built-in agent types
            const KNOWN_AGENTS: &[&str] = &[
                "claude", "codex", "cursor", "gemini", "vscode",
            ];

            match target_cmd {
                TargetCommand::Add { agent, path, name, scope, sync } => {
                    // Validate agent type against built-in + custom adapters
                    if !KNOWN_AGENTS.contains(&agent.as_str())
                        && !config.adapter.contains_key(&agent)
                    {
                        let available: Vec<String> = KNOWN_AGENTS.iter()
                            .map(|s| s.to_string())
                            .chain(config.adapter.keys().cloned())
                            .collect();
                        anyhow::bail!(
                            "unknown agent type '{}'. Available: {}",
                            agent,
                            available.join(", ")
                        );
                    }

                    let target_name = name.unwrap_or_else(|| {
                        format!("{}-{}", agent, scope)
                    });

                    // Check for duplicate name
                    if config.target.iter().any(|t| t.name == target_name) {
                        anyhow::bail!("target '{}' already exists", target_name);
                    }

                    // Resolve path: default based on agent + scope
                    let target_path = if let Some(p) = path {
                        std::path::PathBuf::from(p)
                    } else {
                        // Default: ~/.{agent} for machine scope
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

                    if !cli.dry_run {
                        config.target.push(crate::config::TargetConfig {
                            name: target_name.clone(),
                            agent: agent.clone(),
                            path: target_path,
                            scope,
                            sync: actual_sync,
                        });
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        if cli.dry_run {
                            println!("  (dry run) would add target '{}'", target_name);
                        } else {
                            println!("Added target '{}'", target_name);
                        }
                    }
                    Ok(())
                }
                TargetCommand::Remove { name, force } => {
                    if !config.target.iter().any(|t| t.name == name) {
                        anyhow::bail!("target '{}' not found", name);
                    }

                    let execute = force && !cli.dry_run;
                    if execute {
                        config.target.retain(|t| t.name != name);
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        if execute {
                            println!("Removed target '{}' (installed skills preserved)", name);
                        } else {
                            println!("Would remove target '{}'", name);
                            println!("Use --force to remove");
                        }
                    }
                    Ok(())
                }
                TargetCommand::List => {
                    if cli.json {
                        let entries: Vec<serde_json::Value> = config.target.iter().map(|t| {
                            serde_json::json!({
                                "name": t.name,
                                "agent": t.agent,
                                "path": t.path,
                                "scope": t.scope,
                                "sync": t.sync,
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if config.target.is_empty() {
                        if !cli.quiet {
                            println!("No targets configured. Use `skittle target add` to add one.");
                        }
                        return Ok(());
                    }

                    let rows: Vec<Vec<String>> = config.target.iter().map(|t| {
                        vec![
                            t.name.clone(),
                            t.agent.clone(),
                            t.path.display().to_string(),
                            t.scope.clone(),
                            t.sync.clone(),
                        ]
                    }).collect();

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose,
                    );
                    out.table(
                        &["NAME", "AGENT", "PATH", "SCOPE", "SYNC"],
                        &rows,
                    );
                    Ok(())
                }
                TargetCommand::Show { name } => {
                    let target = config.target.iter()
                        .find(|t| t.name == name)
                        .ok_or_else(|| anyhow::anyhow!("target '{}' not found", name))?;

                    if cli.json {
                        let json = serde_json::json!({
                            "name": target.name,
                            "agent": target.agent,
                            "path": target.path,
                            "scope": target.scope,
                            "sync": target.sync,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose,
                    );
                    out.status("Name", &target.name);
                    out.status("Agent", &target.agent);
                    out.status("Path", &target.path.display().to_string());
                    out.status("Scope", &target.scope);
                    out.status("Sync", &target.sync);

                    // List installed skills if the directory exists
                    let skills_dir = target.path.join("skills");
                    if skills_dir.is_dir() {
                        let mut installed = Vec::new();
                        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                            for entry in entries.flatten() {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    if entry.path().join("SKILL.md").exists() {
                                        installed.push(entry.file_name().to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                        if !installed.is_empty() {
                            installed.sort();
                            out.status("Installed", &installed.len().to_string());
                            out.info("");
                            let tree: Vec<(usize, String)> = installed.into_iter()
                                .map(|s| (0, s))
                                .collect();
                            out.tree(&tree);
                        }
                    }

                    Ok(())
                }
                TargetCommand::Detect { force } => {
                    let home = std::env::var("HOME")
                        .map(std::path::PathBuf::from)
                        .or_else(|_| dirs::home_dir().ok_or(()))
                        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

                    const AGENT_PREFIXES: &[(&str, &str)] = &[
                        ("claude", ".claude"),
                        ("codex", ".codex"),
                        ("cursor", ".cursor"),
                    ];

                    let mut candidates: Vec<(&str, std::path::PathBuf)> = Vec::new();

                    // Scan home and cwd for directories matching agent prefixes
                    let cwd = std::env::current_dir().unwrap_or_default();
                    let dirs_to_scan: Vec<&std::path::Path> = if cwd == home {
                        vec![&home]
                    } else {
                        vec![&home, &cwd]
                    };

                    for scan_dir in &dirs_to_scan {
                        if let Ok(entries) = std::fs::read_dir(scan_dir) {
                            for entry in entries.flatten() {
                                let name = entry.file_name();
                                let name_str = name.to_string_lossy();
                                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    continue;
                                }
                                for (agent, prefix) in AGENT_PREFIXES {
                                    if name_str == *prefix || name_str.starts_with(&format!("{}-", prefix)) {
                                        let path = entry.path();
                                        if !candidates.iter().any(|(_, p)| *p == path) {
                                            candidates.push((agent, path));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    candidates.sort_by(|a, b| a.1.cmp(&b.1));

                    let mut found = Vec::new();
                    for (agent, path) in &candidates {
                        if path.is_dir() {
                            let already = config.target.iter().any(|t| t.path == *path);
                            found.push((agent.to_string(), path.clone(), already));
                        }
                    }

                    if cli.json {
                        let entries: Vec<serde_json::Value> = found.iter().map(|(agent, path, registered)| {
                            serde_json::json!({
                                "agent": agent,
                                "path": path,
                                "registered": registered,
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if found.is_empty() {
                        if !cli.quiet {
                            println!("No agent configurations found.");
                        }
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose,
                    );

                    let mut added = 0;
                    for (agent, path, registered) in &found {
                        if *registered {
                            out.info(&format!("{} at {} (already registered)", agent, path.display()));
                            continue;
                        }

                        let target_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(agent)
                            .trim_start_matches('.')
                            .to_string();
                        let scope = if path.starts_with(&home) { "machine" } else { "repo" };

                        let should_add = if force {
                            true
                        } else {
                            // Prompt user
                            eprint!("Add {} at {} as target '{}'? [y/N] ", agent, path.display(), target_name);
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap_or(0);
                            input.trim().eq_ignore_ascii_case("y")
                        };

                        if should_add {
                            let sync = if scope == "repo" { "explicit" } else { "auto" };
                            config.target.push(crate::config::TargetConfig {
                                name: target_name.clone(),
                                agent: agent.to_string(),
                                path: path.clone(),
                                scope: scope.to_string(),
                                sync: sync.to_string(),
                            });
                            out.success(&format!("Added target '{}'", target_name));
                            added += 1;
                        }
                    }

                    if added > 0 {
                        crate::config::save(&config, config_path_str)?;
                    }

                    Ok(())
                }
            }
        }
        Command::Config { command: config_cmd } => {
            let config = crate::config::load(cli.config.as_deref())?;
            match config_cmd {
                ConfigCommand::Show => {
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&config)?);
                    } else {
                        let path = crate::config::config_path(cli.config.as_deref());
                        println!("Config: {}", path.display());
                        println!();
                        println!("Sources: {}", config.source.len());
                        for s in &config.source {
                            println!("  {} ({})", s.name, s.url);
                        }
                        println!("Targets: {}", config.target.len());
                        for t in &config.target {
                            println!("  {} ({} @ {})", t.name, t.agent, t.path.display());
                        }
                        println!("Adapters: {}", config.adapter.len());
                        for (name, _) in &config.adapter {
                            println!("  {}", name);
                        }
                        println!("Bundles: {}", config.bundle.len());
                        for (name, b) in &config.bundle {
                            println!("  {} ({} skills)", name, b.skills.len());
                        }
                    }
                    Ok(())
                }
                ConfigCommand::Edit => {
                    let path = crate::config::config_path(cli.config.as_deref());
                    if !path.exists() {
                        anyhow::bail!("No config found. Run `skittle init` first.");
                    }
                    let editor = std::env::var("EDITOR")
                        .or_else(|_| std::env::var("VISUAL"))
                        .unwrap_or_else(|_| "vi".to_string());
                    let status = std::process::Command::new(&editor)
                        .arg(&path)
                        .status()?;
                    if !status.success() {
                        anyhow::bail!("editor exited with {}", status);
                    }
                    Ok(())
                }
            }
        }
    }
}

/// Recursively copy a directory's contents.
/// Record provenance for an applied skill.
fn record_provenance(
    reg: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    tc: &crate::config::TargetConfig,
    src_name: &str,
    plug_name: &str,
    s: &crate::registry::RegisteredSkill,
) {
    let origin = s.path.strip_prefix(data_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| s.path.display().to_string());
    let target_map = reg.installed.entry(tc.name.clone()).or_default();
    target_map.insert(s.name.clone(), crate::registry::InstalledSkill {
        source: src_name.to_string(),
        plugin: plug_name.to_string(),
        skill: s.name.clone(),
        origin,
    });
}

/// Action chosen by user in interactive conflict resolution.
enum ConflictAction {
    Skip,
    Overwrite,
    ForceAll,
    Quit,
}

/// Prompt the user to resolve a conflict for a changed skill.
fn prompt_conflict(
    skill: &crate::registry::RegisteredSkill,
    adapter: &crate::target::Adapter,
    target_path: &std::path::Path,
) -> anyhow::Result<ConflictAction> {
    eprintln!();
    eprintln!("  {} — CHANGED", skill.name);
    eprintln!();
    eprint!("    [s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit: ");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        match choice.as_str() {
            "s" => return Ok(ConflictAction::Skip),
            "o" => return Ok(ConflictAction::Overwrite),
            "f" => return Ok(ConflictAction::ForceAll),
            "q" => return Ok(ConflictAction::Quit),
            "d" => {
                show_skill_diff(skill, adapter, target_path)?;
                eprintln!();
                eprint!("    [s]kip  [o]verwrite  [q]uit: ");
                // After diff, loop again for s/o/q only
                let mut input2 = String::new();
                std::io::stdin().read_line(&mut input2)?;
                let choice2 = input2.trim().to_lowercase();
                match choice2.as_str() {
                    "s" => return Ok(ConflictAction::Skip),
                    "o" => return Ok(ConflictAction::Overwrite),
                    "q" => return Ok(ConflictAction::Quit),
                    _ => {
                        eprint!("    Invalid choice. [s]kip  [o]verwrite  [q]uit: ");
                        continue;
                    }
                }
            }
            _ => {
                eprint!("    Invalid choice. [s]kip  [o]verwrite  [d]iff  [f]orce-all  [q]uit: ");
                continue;
            }
        }
    }
}

/// Display a unified diff of all files in a skill directory.
fn show_skill_diff(
    skill: &crate::registry::RegisteredSkill,
    adapter: &crate::target::Adapter,
    target_path: &std::path::Path,
) -> anyhow::Result<()> {
    let pairs = adapter.skill_file_pairs(skill, target_path)?;

    for (label, src_path, dst_path) in &pairs {
        let src_content = if src_path.exists() {
            std::fs::read_to_string(src_path).unwrap_or_default()
        } else {
            String::new()
        };
        let dst_content = if dst_path.exists() {
            std::fs::read_to_string(dst_path).unwrap_or_default()
        } else {
            String::new()
        };

        if src_content == dst_content {
            continue;
        }

        eprintln!();
        eprintln!("    === {} ===", label);

        let diff = similar::TextDiff::from_lines(&dst_content, &src_content);
        for hunk in diff.unified_diff().header("installed", "source").iter_hunks() {
            eprint!("    {}", hunk);
        }
    }

    Ok(())
}

/// Print the apply summary line.
fn print_apply_summary(new_count: usize, updated_count: usize, unchanged_count: usize, conflict_skipped: usize, quiet: bool) {
    if quiet {
        return;
    }
    let applied = new_count + updated_count;
    let mut msg = format!("Applied {} skill(s) ({} new, {} updated), skipped {} unchanged.", applied, new_count, updated_count, unchanged_count);
    if conflict_skipped > 0 {
        msg.push_str(&format!(" {} conflict skipped.", conflict_skipped));
    }
    println!("{}", msg);
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Generate .claude-plugin/marketplace.json from the plugins/ directory.
fn generate_marketplace(data_dir: &std::path::Path) -> anyhow::Result<()> {
    let plugins_dir = data_dir.join("plugins");
    let mut plugins = Vec::new();

    if plugins_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(&plugins_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.') {
                continue;
            }

            // Try to read plugin.json for metadata
            let plugin_json = entry.path().join(".claude-plugin/plugin.json");
            let (name, description) = if plugin_json.exists() {
                if let Ok(manifest) = crate::source::manifest::load_plugin_manifest(&plugin_json) {
                    (manifest.name, manifest.description)
                } else {
                    (dir_name.clone(), None)
                }
            } else {
                (dir_name.clone(), None)
            };

            let mut plugin_entry = serde_json::json!({
                "name": name,
                "source": format!("./plugins/{}", dir_name),
            });
            if let Some(desc) = description {
                plugin_entry["description"] = serde_json::Value::String(desc);
            }
            plugins.push(plugin_entry);
        }
    }

    let marketplace = serde_json::json!({
        "name": "skittle-marketplace",
        "plugins": plugins,
    });

    let cp_dir = data_dir.join(".claude-plugin");
    std::fs::create_dir_all(&cp_dir)?;
    std::fs::write(
        cp_dir.join("marketplace.json"),
        serde_json::to_string_pretty(&marketplace)?,
    )?;

    Ok(())
}
