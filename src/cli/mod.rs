use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "skittle",
    about = "Agent skill manager — source, cache, and install skills across coding agents",
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

    /// Color output control
    #[arg(long, global = true, value_name = "WHEN", default_value = "auto")]
    pub color: ColorWhen,

    /// Path to config file
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<String>,
}

#[derive(Clone, ValueEnum)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize skittle configuration
    Init,

    /// Install skills to targets
    Install {
        /// Install all configured skills
        #[arg(long)]
        all: bool,

        /// Install a specific skill (plugin/skill)
        #[arg(long, value_name = "SKILL")]
        skill: Option<String>,

        /// Install all skills from a plugin
        #[arg(long, value_name = "PLUGIN")]
        plugin: Option<String>,

        /// Install a bundle of skills
        #[arg(long, value_name = "BUNDLE")]
        bundle: Option<String>,

        /// Target to install to
        #[arg(long, value_name = "TARGET")]
        target: Option<String>,
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
    },

    /// Show current status
    Status,

    /// Manage skill sources
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },

    /// Manage plugins
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },

    /// Manage skills
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },

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

    /// Manage local cache
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
}

#[derive(Subcommand)]
pub enum SourceCommand {
    /// Add a skill source
    Add {
        /// URL or path to the source
        url: String,

        /// Name for this source
        #[arg(long)]
        name: Option<String>,
    },
    /// Remove a skill source
    Remove {
        /// Source name
        name: String,

        /// Force removal even if skills are installed
        #[arg(long)]
        force: bool,
    },
    /// List all sources
    List,
    /// Show source details
    Show {
        /// Source name
        name: String,
    },
    /// Update source(s) from remote
    Update {
        /// Source name (omit to update all)
        name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PluginCommand {
    /// List plugins
    List {
        /// Filter by source
        #[arg(long)]
        source: Option<String>,
    },
    /// Show plugin details
    Show {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SkillCommand {
    /// List skills
    List {
        /// Filter by source
        #[arg(long)]
        source: Option<String>,

        /// Filter by plugin
        #[arg(long)]
        plugin: Option<String>,
    },
    /// Show skill details
    Show {
        /// Skill identity (plugin/skill or source:plugin/skill)
        identity: String,
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
    /// List all bundles
    List,
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
    /// Swap active bundle (uninstall from, install to)
    Swap {
        /// Bundle to uninstall
        from: String,

        /// Bundle to install
        to: String,

        /// Target for the swap
        #[arg(long)]
        target: Option<String>,
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
    },
    /// List all targets
    List,
    /// Show target details
    Show {
        /// Target name
        name: String,
    },
    /// Detect agent installations
    Detect,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Open config in editor
    Edit,
}

#[derive(Subcommand)]
pub enum CacheCommand {
    /// Show cache information
    Show,
    /// Clean cached data
    Clean,
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init => {
            let path = crate::config::config_path(cli.config.as_deref());
            if path.exists() {
                if !cli.quiet {
                    println!("Config already exists at {}. Use `skittle config edit` to modify.", path.display());
                }
                return Ok(());
            }
            // Create config directory and write default config
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Also create data directory
            let data = crate::config::data_dir();
            std::fs::create_dir_all(&data)?;

            let default_config = r#"# Skittle configuration
# See: skittle --help

# Sources — where skills come from
# [[source]]
# name = "my-skills"
# url = "~/dev/my-skills"
# type = "local"

# [[source]]
# name = "community"
# url = "https://github.com/org/skills.git"
# type = "git"

# Targets — where skills get installed
# [[target]]
# name = "claude-machine"
# agent = "claude"
# path = "~/.claude"
# scope = "machine"
# sync = "auto"

# Custom adapters
# [adapter.my-agent]
# skill_dir = "prompts/{name}"
# skill_file = "SKILL.md"
# format = "agentskills"
# copy_dirs = ["scripts"]

# Bundles — named groups of skills
# [bundle.work]
# skills = ["my-plugin/explore", "my-plugin/apply"]
"#;
            std::fs::write(&path, default_config)?;
            if !cli.quiet {
                println!("Initialized skittle config at {}", path.display());
            }
            Ok(())
        }
        Command::Install { all, skill, plugin, bundle, target } => {
            if !all && skill.is_none() && plugin.is_none() && bundle.is_none() {
                eprintln!("error: install requires --all, --skill, --plugin, or --bundle");
                std::process::exit(2);
            }

            let config_path_str = cli.config.as_deref();
            let config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            // Determine which targets to install to
            let targets: Vec<&crate::config::TargetConfig> = if let Some(ref t) = target {
                let tc = config.target.iter()
                    .find(|tc| tc.name == *t)
                    .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                vec![tc]
            } else {
                // Install to auto-sync targets
                config.target.iter().filter(|t| t.sync == "auto").collect()
            };

            if targets.is_empty() {
                anyhow::bail!("no targets configured. Use `skittle target add` first.");
            }

            // Collect skills to install
            let mut skills_to_install: Vec<&crate::registry::RegisteredSkill> = Vec::new();

            if all {
                for src in &registry.sources {
                    for p in &src.plugins {
                        for s in &p.skills {
                            skills_to_install.push(s);
                        }
                    }
                }
            }

            if let Some(ref skill_id) = skill {
                let (_, _, s) = registry.find_skill(skill_id)?;
                skills_to_install.push(s);
            }

            if let Some(ref plugin_name) = plugin {
                let (_, p) = registry.find_plugin(plugin_name)
                    .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_name))?;
                for s in &p.skills {
                    skills_to_install.push(s);
                }
            }

            if let Some(ref bundle_name) = bundle {
                let bundle_cfg = config.bundle.get(bundle_name)
                    .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", bundle_name))?;
                for skill_id in &bundle_cfg.skills {
                    let (_, _, s) = registry.find_skill(skill_id)?;
                    skills_to_install.push(s);
                }
            }

            // Install to each target
            let mut installed_count = 0;
            for tc in &targets {
                let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
                for s in &skills_to_install {
                    if cli.dry_run {
                        if !cli.quiet {
                            println!("  (dry run) {} → {}", s.name, tc.name);
                        }
                    } else {
                        adapter.install_skill(s, &tc.path)?;
                    }
                    installed_count += 1;
                }
            }

            // Track active bundle if installing a bundle
            if let Some(ref bundle_name) = bundle {
                if !cli.dry_run {
                    let mut reg = registry;
                    // Store active bundle per target in registry
                    // We'll use a simple field on the registry
                    for tc in &targets {
                        reg.set_active_bundle(&tc.name, bundle_name);
                    }
                    crate::registry::save_registry(&reg, &data_dir)?;
                }
            }

            if !cli.quiet {
                println!("Installed {} skill(s) to {} target(s)", installed_count / targets.len().max(1), targets.len());
            }
            Ok(())
        }
        Command::Uninstall { skill, plugin, bundle, target } => {
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

            for tc in &targets {
                let adapter = crate::target::resolve_adapter(tc, &config.adapter)?;
                for name in &skill_names {
                    if cli.dry_run {
                        if !cli.quiet {
                            println!("  (dry run) uninstall {} from {}", name, tc.name);
                        }
                    } else {
                        adapter.uninstall_skill(name, &tc.path)?;
                    }
                }
            }

            if !cli.quiet {
                println!("Uninstalled {} skill(s) from {} target(s)", skill_names.len(), targets.len());
            }
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
                    "active_bundles": registry.active_bundles,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(
                cli.json, cli.quiet, cli.verbose, &cli.color,
            );
            out.status("Sources", &config.source.len().to_string());
            out.status("Targets", &config.target.len().to_string());
            out.status("Plugins", &registry.sources.iter().flat_map(|s| &s.plugins).count().to_string());
            out.status("Skills", &total_skills.to_string());
            out.status("Installed", &total_installed.to_string());
            out.status("Bundles", &config.bundle.len().to_string());

            if !registry.active_bundles.is_empty() {
                out.info("");
                out.info("Active bundles:");
                for (target, bundle) in &registry.active_bundles {
                    out.info(&format!("  {} → {}", target, bundle));
                }
            }

            Ok(())
        }
        Command::Source { command: source_cmd } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();

            match source_cmd {
                SourceCommand::Add { url, name } => {
                    let source_url = crate::source::SourceUrl::parse(&url)?;
                    let source_name = name.unwrap_or_else(|| source_url.default_name());

                    // Check for duplicate name
                    if config.source.iter().any(|s| s.name == source_name) {
                        anyhow::bail!(
                            "source '{}' already exists. Use --name to choose a different alias.",
                            source_name
                        );
                    }

                    let cache_path = crate::config::cache_dir().join(&source_name);

                    if !cli.dry_run {
                        // Fetch source content into cache
                        crate::source::fetch::fetch(&source_url, &cache_path)?;

                        // Detect structure
                        let structure = crate::source::detect::detect(&cache_path)?;

                        // Normalize into registry model
                        let registered = crate::source::normalize::normalize(
                            &source_name, &cache_path, &structure,
                        )?;

                        // Update registry
                        let mut registry = crate::registry::load_registry(&data_dir)?;
                        registry.sources.retain(|s| s.name != source_name);
                        registry.sources.push(registered);
                        crate::registry::save_registry(&registry, &data_dir)?;

                        // Update config
                        config.source.push(crate::config::SourceConfig {
                            name: source_name.clone(),
                            url: source_url.url_string(),
                            source_type: source_url.source_type().to_string(),
                        });
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        println!("Added source '{}'", source_name);
                    }
                    Ok(())
                }
                SourceCommand::Remove { name, force } => {
                    if !config.source.iter().any(|s| s.name == name) {
                        anyhow::bail!("source '{}' not found", name);
                    }

                    if !cli.dry_run {
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
                        println!("Removed source '{}'", name);
                    }
                    let _ = force; // acknowledged for future installed-skill check
                    Ok(())
                }
                SourceCommand::List => {
                    let registry = crate::registry::load_registry(&data_dir)?;

                    if cli.json {
                        let entries: Vec<serde_json::Value> = config.source.iter().map(|s| {
                            let plugin_count = registry.sources.iter()
                                .find(|r| r.name == s.name)
                                .map(|r| r.plugins.len())
                                .unwrap_or(0);
                            serde_json::json!({
                                "name": s.name,
                                "url": s.url,
                                "type": s.source_type,
                                "plugins": plugin_count,
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if config.source.is_empty() {
                        if !cli.quiet {
                            println!("No sources registered. Use `skittle source add` to add one.");
                        }
                        return Ok(());
                    }

                    let rows: Vec<Vec<String>> = config.source.iter().map(|s| {
                        let plugin_count = registry.sources.iter()
                            .find(|r| r.name == s.name)
                            .map(|r| r.plugins.len())
                            .unwrap_or(0);
                        vec![
                            s.name.clone(),
                            s.url.clone(),
                            s.source_type.clone(),
                            plugin_count.to_string(),
                        ]
                    }).collect();

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.table(
                        &["NAME", "URL", "TYPE", "PLUGINS"],
                        &rows,
                    );
                    Ok(())
                }
                SourceCommand::Show { name } => {
                    let source_cfg = config.source.iter()
                        .find(|s| s.name == name)
                        .ok_or_else(|| anyhow::anyhow!("source '{}' not found", name))?;

                    let registry = crate::registry::load_registry(&data_dir)?;
                    let registered = registry.sources.iter().find(|s| s.name == name);

                    if cli.json {
                        let json = serde_json::json!({
                            "name": source_cfg.name,
                            "url": source_cfg.url,
                            "type": source_cfg.source_type,
                            "plugins": registered.map(|r| {
                                r.plugins.iter().map(|p| {
                                    serde_json::json!({
                                        "name": p.name,
                                        "version": p.version,
                                        "description": p.description,
                                        "skills": p.skills.iter().map(|s| {
                                            serde_json::json!({
                                                "name": s.name,
                                                "description": s.description,
                                            })
                                        }).collect::<Vec<_>>(),
                                    })
                                }).collect::<Vec<_>>()
                            }).unwrap_or_default(),
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.status("Name", &source_cfg.name);
                    out.status("URL", &source_cfg.url);
                    out.status("Type", &source_cfg.source_type);

                    if let Some(reg) = registered {
                        out.info("");
                        let mut tree_entries = Vec::new();
                        for plugin in &reg.plugins {
                            let plugin_label = if let Some(v) = &plugin.version {
                                format!("{} (v{})", plugin.name, v)
                            } else {
                                plugin.name.clone()
                            };
                            tree_entries.push((0, plugin_label));
                            for skill in &plugin.skills {
                                let skill_label = if let Some(d) = &skill.description {
                                    format!("{} — {}", skill.name, d)
                                } else {
                                    skill.name.clone()
                                };
                                tree_entries.push((1, skill_label));
                            }
                        }
                        out.tree(&tree_entries);
                    }

                    Ok(())
                }
                SourceCommand::Update { name } => {
                    let registry = crate::registry::load_registry(&data_dir)?;

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

                        // For local sources, remove cache and re-copy
                        // For git sources, update_git handles fetch + reset
                        match &source_url {
                            crate::source::SourceUrl::Local(path) => {
                                if cache_path.exists() {
                                    std::fs::remove_dir_all(&cache_path)?;
                                }
                                if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path) {
                                    errors.push(format!("{}: {}", src.name, e));
                                    continue;
                                }
                                let _ = path; // used via source_url
                            }
                            crate::source::SourceUrl::Git(_) => {
                                if cache_path.exists() {
                                    if let Err(e) = crate::source::fetch::update_git(&cache_path) {
                                        errors.push(format!("{}: {}", src.name, e));
                                        continue;
                                    }
                                } else {
                                    if let Err(e) = crate::source::fetch::fetch(&source_url, &cache_path) {
                                        errors.push(format!("{}: {}", src.name, e));
                                        continue;
                                    }
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
            }
        }
        Command::Plugin { command: plugin_cmd } => {
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            match plugin_cmd {
                PluginCommand::List { source } => {
                    // Collect all plugins, optionally filtered by source
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut json_entries: Vec<serde_json::Value> = Vec::new();

                    for src in &registry.sources {
                        if let Some(ref filter) = source {
                            if &src.name != filter {
                                continue;
                            }
                        }
                        for plugin in &src.plugins {
                            rows.push(vec![
                                plugin.name.clone(),
                                src.name.clone(),
                                plugin.version.clone().unwrap_or_default(),
                                plugin.skills.len().to_string(),
                            ]);
                            json_entries.push(serde_json::json!({
                                "name": plugin.name,
                                "source": src.name,
                                "version": plugin.version,
                                "description": plugin.description,
                                "skills": plugin.skills.len(),
                            }));
                        }
                    }

                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&json_entries)?);
                        return Ok(());
                    }

                    if rows.is_empty() {
                        if !cli.quiet {
                            println!("No plugins found. Use `skittle source add` to add a source.");
                        }
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.table(
                        &["PLUGIN", "SOURCE", "VERSION", "SKILLS"],
                        &rows,
                    );
                    Ok(())
                }
                PluginCommand::Show { name } => {
                    // Find plugin by name across all sources
                    let mut found = None;
                    let mut found_source = "";
                    for src in &registry.sources {
                        for plugin in &src.plugins {
                            if plugin.name == name {
                                found = Some(plugin);
                                found_source = &src.name;
                                break;
                            }
                        }
                        if found.is_some() { break; }
                    }

                    let plugin = found
                        .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", name))?;

                    if cli.json {
                        let json = serde_json::json!({
                            "name": plugin.name,
                            "source": found_source,
                            "version": plugin.version,
                            "description": plugin.description,
                            "skills": plugin.skills.iter().map(|s| {
                                serde_json::json!({
                                    "name": s.name,
                                    "description": s.description,
                                })
                            }).collect::<Vec<_>>(),
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.status("Plugin", &plugin.name);
                    out.status("Source", found_source);
                    if let Some(v) = &plugin.version {
                        out.status("Version", v);
                    }
                    if let Some(d) = &plugin.description {
                        out.status("Description", d);
                    }
                    out.status("Skills", &plugin.skills.len().to_string());

                    if !plugin.skills.is_empty() {
                        out.info("");
                        let tree_entries: Vec<(usize, String)> = plugin.skills.iter().map(|s| {
                            let label = if let Some(d) = &s.description {
                                format!("{} — {}", s.name, d)
                            } else {
                                s.name.clone()
                            };
                            (0, label)
                        }).collect();
                        out.tree(&tree_entries);
                    }

                    Ok(())
                }
            }
        }
        Command::Skill { command: skill_cmd } => {
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            match skill_cmd {
                SkillCommand::List { source, plugin } => {
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut json_entries: Vec<serde_json::Value> = Vec::new();

                    for src in &registry.sources {
                        if let Some(ref filter) = source {
                            if &src.name != filter {
                                continue;
                            }
                        }
                        for p in &src.plugins {
                            if let Some(ref filter) = plugin {
                                if &p.name != filter {
                                    continue;
                                }
                            }
                            for skill in &p.skills {
                                rows.push(vec![
                                    skill.name.clone(),
                                    p.name.clone(),
                                    src.name.clone(),
                                    skill.description.clone().unwrap_or_default(),
                                ]);
                                json_entries.push(serde_json::json!({
                                    "name": skill.name,
                                    "plugin": p.name,
                                    "source": src.name,
                                    "description": skill.description,
                                }));
                            }
                        }
                    }

                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&json_entries)?);
                        return Ok(());
                    }

                    if rows.is_empty() {
                        if !cli.quiet {
                            println!("No skills found. Use `skittle source add` to add a source.");
                        }
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.table(
                        &["SKILL", "PLUGIN", "SOURCE", "DESCRIPTION"],
                        &rows,
                    );
                    Ok(())
                }
                SkillCommand::Show { identity } => {
                    let (source_name, plugin_name, skill) = registry.find_skill(&identity)?;

                    if cli.json {
                        let json = serde_json::json!({
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
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.status("Skill", &skill.name);
                    out.status("Plugin", plugin_name);
                    out.status("Source", source_name);
                    if let Some(d) = &skill.description {
                        out.status("Description", d);
                    }
                    if cli.verbose {
                        out.status("Path", &skill.path.display().to_string());
                    }

                    Ok(())
                }
            }
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

                    // Check if active
                    let registry = crate::registry::load_registry(&data_dir)?;
                    let is_active = registry.active_bundles.values().any(|b| b == &name);
                    if is_active && !force {
                        anyhow::bail!(
                            "bundle '{}' is active on a target. Use --force to delete.",
                            name
                        );
                    }

                    config.bundle.remove(&name);
                    crate::config::save(&config, config_path_str)?;

                    // Clear from active bundles
                    if is_active {
                        let mut reg = registry;
                        reg.active_bundles.retain(|_, v| v != &name);
                        crate::registry::save_registry(&reg, &data_dir)?;
                    }

                    if !cli.quiet {
                        println!("Deleted bundle '{}'", name);
                    }
                    Ok(())
                }
                BundleCommand::List => {
                    let registry = crate::registry::load_registry(&data_dir)?;

                    if cli.json {
                        let entries: Vec<serde_json::Value> = config.bundle.iter().map(|(name, b)| {
                            let active_targets: Vec<&str> = registry.active_bundles.iter()
                                .filter(|(_, v)| v.as_str() == name)
                                .map(|(k, _)| k.as_str())
                                .collect();
                            serde_json::json!({
                                "name": name,
                                "skills": b.skills.len(),
                                "active_targets": active_targets,
                            })
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if config.bundle.is_empty() {
                        if !cli.quiet {
                            println!("No bundles configured. Use `skittle bundle create` to create one.");
                        }
                        return Ok(());
                    }

                    let rows: Vec<Vec<String>> = config.bundle.iter().map(|(name, b)| {
                        let active: Vec<&str> = registry.active_bundles.iter()
                            .filter(|(_, v)| v.as_str() == name)
                            .map(|(k, _)| k.as_str())
                            .collect();
                        vec![
                            name.clone(),
                            b.skills.len().to_string(),
                            if active.is_empty() { String::new() } else { active.join(", ") },
                        ]
                    }).collect();

                    let out = crate::output::Output::from_flags(
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    out.table(&["BUNDLE", "SKILLS", "ACTIVE ON"], &rows);
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
                        cli.json, cli.quiet, cli.verbose, &cli.color,
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

                    for skill_id in &skills {
                        // Validate skill exists
                        registry.find_skill(skill_id)?;
                        // Add if not already present
                        if !bundle.skills.contains(skill_id) {
                            bundle.skills.push(skill_id.clone());
                        }
                    }

                    crate::config::save(&config, config_path_str)?;
                    if !cli.quiet {
                        println!("Added {} skill(s) to bundle '{}'", skills.len(), name);
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
                BundleCommand::Swap { from, to, target } => {
                    let config_for_install = config.clone();
                    let mut registry = crate::registry::load_registry(&data_dir)?;

                    // Validate both bundles exist
                    let from_bundle = config.bundle.get(&from)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", from))?
                        .clone();
                    let to_bundle = config.bundle.get(&to)
                        .ok_or_else(|| anyhow::anyhow!("bundle '{}' not found", to))?
                        .clone();

                    // Determine targets
                    let targets: Vec<&crate::config::TargetConfig> = if let Some(ref t) = target {
                        let tc = config_for_install.target.iter()
                            .find(|tc| tc.name == *t)
                            .ok_or_else(|| anyhow::anyhow!("target '{}' not found", t))?;
                        vec![tc]
                    } else {
                        config_for_install.target.iter().filter(|t| t.sync == "auto").collect()
                    };

                    for tc in &targets {
                        let adapter = crate::target::resolve_adapter(tc, &config_for_install.adapter)?;

                        if !cli.dry_run {
                            // Uninstall 'from' skills
                            for skill_id in &from_bundle.skills {
                                if let Ok((_, _, s)) = registry.find_skill(skill_id) {
                                    adapter.uninstall_skill(&s.name, &tc.path)?;
                                }
                            }
                            // Install 'to' skills
                            for skill_id in &to_bundle.skills {
                                let (_, _, s) = registry.find_skill(skill_id)?;
                                adapter.install_skill(s, &tc.path)?;
                            }
                            // Update active bundle
                            registry.set_active_bundle(&tc.name, &to);
                        } else if !cli.quiet {
                            println!("  (dry run) swap {} → {} on {}", from, to, tc.name);
                        }
                    }

                    if !cli.dry_run {
                        crate::registry::save_registry(&registry, &data_dir)?;
                    }

                    if !cli.quiet {
                        println!("Swapped bundle '{}' → '{}' on {} target(s)", from, to, targets.len());
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
                        println!("Added target '{}'", target_name);
                    }
                    Ok(())
                }
                TargetCommand::Remove { name } => {
                    if !config.target.iter().any(|t| t.name == name) {
                        anyhow::bail!("target '{}' not found", name);
                    }

                    if !cli.dry_run {
                        config.target.retain(|t| t.name != name);
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        println!("Removed target '{}' (installed skills preserved)", name);
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
                        cli.json, cli.quiet, cli.verbose, &cli.color,
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
                        cli.json, cli.quiet, cli.verbose, &cli.color,
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
                TargetCommand::Detect => {
                    let home = std::env::var("HOME")
                        .map(std::path::PathBuf::from)
                        .or_else(|_| dirs::home_dir().ok_or(()))
                        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

                    let candidates = vec![
                        ("claude", home.join(".claude")),
                        ("codex", home.join(".codex")),
                        ("cursor", home.join(".cursor")),
                    ];

                    // Also check current directory
                    let cwd = std::env::current_dir().unwrap_or_default();
                    let local_candidates = vec![
                        ("claude", cwd.join(".claude")),
                        ("codex", cwd.join(".codex")),
                    ];

                    let mut found = Vec::new();
                    for (agent, path) in candidates.iter().chain(local_candidates.iter()) {
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
                        cli.json, cli.quiet, cli.verbose, &cli.color,
                    );
                    for (agent, path, registered) in &found {
                        let status = if *registered { " (registered)" } else { "" };
                        out.info(&format!(
                            "Found {} at {}{}",
                            agent,
                            path.display(),
                            status
                        ));
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
        Command::Cache { command } => {
            let data_dir = crate::config::data_dir();
            let cache_dir = crate::config::cache_dir();
            let output = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose, &cli.color);

            match command {
                CacheCommand::Show => {
                    if cli.json {
                        let mut sources = Vec::new();
                        let mut total_size: u64 = 0;
                        if cache_dir.is_dir() {
                            for entry in std::fs::read_dir(&cache_dir)?.flatten() {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    let size = dir_size(&entry.path());
                                    total_size += size;
                                    sources.push(serde_json::json!({
                                        "name": entry.file_name().to_string_lossy(),
                                        "path": entry.path(),
                                        "size_bytes": size,
                                    }));
                                }
                            }
                        }
                        output.json(&serde_json::json!({
                            "cache_path": cache_dir,
                            "total_size_bytes": total_size,
                            "total_size_human": format_size(total_size),
                            "sources": sources,
                        }));
                    } else {
                        output.header("Cache");
                        output.status("Path", &cache_dir.display().to_string());

                        let mut total_size: u64 = 0;
                        let mut rows = Vec::new();
                        if cache_dir.is_dir() {
                            for entry in std::fs::read_dir(&cache_dir)?.flatten() {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    let size = dir_size(&entry.path());
                                    total_size += size;
                                    rows.push(vec![
                                        entry.file_name().to_string_lossy().to_string(),
                                        format_size(size),
                                    ]);
                                }
                            }
                        }
                        output.status("Total size", &format_size(total_size));
                        if !rows.is_empty() {
                            println!();
                            output.table(&["Source", "Size"], &rows);
                        }
                    }
                    Ok(())
                }
                CacheCommand::Clean => {
                    if cli.dry_run {
                        output.info("Would clean cache and registry");
                        if cache_dir.is_dir() {
                            let size = dir_size(&cache_dir);
                            output.info(&format!("Would free {}", format_size(size)));
                        }
                        return Ok(());
                    }

                    let mut freed: u64 = 0;
                    if cache_dir.is_dir() {
                        freed = dir_size(&cache_dir);
                        std::fs::remove_dir_all(&cache_dir)
                            .with_context(|| format!("failed to remove {}", cache_dir.display()))?;
                        std::fs::create_dir_all(&cache_dir)?;
                    }

                    // Clear registry
                    let registry = crate::registry::Registry::default();
                    crate::registry::save_registry(&registry, &data_dir)?;

                    output.success(&format!("Cache cleaned, freed {}", format_size(freed)));
                    Ok(())
                }
            }
        }
    }
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
