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
        Command::Install { all, skill, plugin, bundle, target: _ } => {
            if !all && skill.is_none() && plugin.is_none() && bundle.is_none() {
                eprintln!("error: install requires --all, --skill, --plugin, or --bundle");
                std::process::exit(2);
            }
            eprintln!("skittle: install not yet implemented");
            std::process::exit(1);
        }
        Command::Uninstall { skill, plugin, bundle, target: _ } => {
            if skill.is_none() && plugin.is_none() && bundle.is_none() {
                eprintln!("error: uninstall requires --skill, --plugin, or --bundle");
                std::process::exit(2);
            }
            eprintln!("skittle: uninstall not yet implemented");
            std::process::exit(1);
        }
        Command::Status => {
            eprintln!("skittle: status not yet implemented");
            std::process::exit(1);
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

                    let cache_path = data_dir.join("sources").join(&source_name);

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
                        let cache_path = data_dir.join("sources").join(&name);
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

                        let cache_path = data_dir.join("sources").join(&src.name);

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
        Command::Skill { command: _ } => {
            eprintln!("skittle: skill not yet implemented");
            std::process::exit(1);
        }
        Command::Bundle { command: _ } => {
            eprintln!("skittle: bundle not yet implemented");
            std::process::exit(1);
        }
        Command::Target { command: _ } => {
            eprintln!("skittle: target not yet implemented");
            std::process::exit(1);
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
        Command::Cache { command: _ } => {
            eprintln!("skittle: cache not yet implemented");
            std::process::exit(1);
        }
    }
}
