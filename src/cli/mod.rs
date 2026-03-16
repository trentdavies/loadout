pub mod args;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "loadout",
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
    /// Initialize loadout configuration
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

        /// Interactive fuzzy finder with skill preview (requires fzf)
        #[arg(long)]
        fzf: bool,
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

    /// Show current status
    Status,

    /// Manage skill kits
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Kit {
        #[command(subcommand)]
        command: KitCommand,
    },

    /// Manage agents
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },

    /// Manage configuration
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Generate shell completions
    #[command(after_long_help = crate::completions::AFTER_HELP)]
    Completions {
        /// Shell to generate completions for
        shell: CompletionShell,

        /// Auto-install to the standard location for your shell
        #[arg(long)]
        install: bool,
    },

    /// Output completion values (used internally by shell scripts)
    #[command(name = "_complete", hide = true)]
    Complete {
        /// Completion type: sources, plugins, skills, agents, kits
        kind: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
pub enum KitCommand {
    /// Create a new kit, optionally seeding it with skills
    Create {
        /// Kit name
        name: String,

        /// Skills or glob patterns to add (e.g. "dev", "hashico*", "openai:openai-skills/skill-creator")
        skills: Vec<String>,
    },
    /// Delete a kit
    Delete {
        /// Kit name
        name: String,

        /// Force deletion
        #[arg(long)]
        force: bool,
    },
    /// List all kits, optionally filtered by name pattern
    List {
        /// Name patterns to filter by (glob supported)
        patterns: Vec<String>,
    },
    /// Show kit details
    Show {
        /// Kit name
        name: String,
    },
    /// Add skills to a kit
    Add {
        /// Kit name
        name: String,

        /// Skills to add (plugin/skill)
        #[arg(required = true)]
        skills: Vec<String>,
    },
    /// Remove skills from a kit
    Drop {
        /// Kit name
        name: String,

        /// Skills to remove (plugin/skill)
        #[arg(required = true)]
        skills: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Add an agent
    Add {
        /// Agent type (claude, codex, cursor, etc.)
        agent: String,

        /// Path to agent directory
        path: Option<String>,

        /// Name for this agent
        #[arg(long)]
        name: Option<String>,

        /// Scope: machine or repo
        #[arg(long, default_value = "machine")]
        scope: String,

        /// Sync mode: auto or explicit
        #[arg(long, default_value = "auto")]
        sync: String,
    },
    /// Remove an agent
    Remove {
        /// Agent name
        name: String,

        /// Actually perform the removal (default is dry run)
        #[arg(long)]
        force: bool,
    },
    /// List all agents
    List,
    /// Show agent details
    Show {
        /// Agent name
        name: String,
    },
    /// Detect agent installations and prompt to add them
    Detect {
        /// Automatically add all detected agents without prompting
        #[arg(long)]
        force: bool,
    },

    /// Equip skills to agent(s)
    Equip {
        /// Glob patterns matching skills (e.g. "legal/*", "*")
        patterns: Vec<String>,

        /// Agent name(s) to equip to (repeatable; defaults to auto-sync agents)
        #[arg(short, long, num_args = 1..)]
        agent: Option<Vec<String>>,

        /// Equip to all configured agents
        #[arg(long, conflicts_with = "agent")]
        all: bool,

        /// Equip a saved kit by name
        #[arg(short, long)]
        kit: Option<String>,

        /// Save the resolved skill set as the kit given by --kit
        #[arg(short, long)]
        save: bool,

        /// Overwrite changed skills without prompting
        #[arg(short, long)]
        force: bool,

        /// Interactively resolve conflicts for changed skills
        #[arg(short, long)]
        interactive: bool,
    },

    /// Unequip skills from agent(s)
    Unequip {
        /// Glob patterns matching skills (e.g. "legal/*", "*")
        patterns: Vec<String>,

        /// Agent name(s) to unequip from (repeatable; defaults to auto-sync agents)
        #[arg(short, long, num_args = 1..)]
        agent: Option<Vec<String>>,

        /// Unequip from all configured agents
        #[arg(long, conflicts_with = "agent")]
        all: bool,

        /// Unequip a saved kit by name
        #[arg(short, long)]
        kit: Option<String>,

        /// Execute removal (default is preview)
        #[arg(short, long)]
        force: bool,
    },

    /// Collect skills from an agent back to source
    Collect {
        /// Agent to collect from
        #[arg(long, value_name = "AGENT")]
        agent: String,

        /// Skill name to collect
        #[arg(long, value_name = "SKILL")]
        skill: Option<String>,

        /// Adopt skill into plugins/ (make it yours)
        #[arg(long)]
        adopt: bool,

        /// Auto-adopt all untracked skills without prompting
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
    // Try to extract host and path from the URL
    let (host, path) = if let Some(rest) = url.strip_prefix("git@") {
        // git@github.com:org/repo.git → ("github.com", "org/repo.git")
        let mut parts = rest.splitn(2, ':');
        let h = parts.next().unwrap_or("");
        let p = parts.next().unwrap_or("");
        (h.to_string(), p.to_string())
    } else if let Some(after_scheme) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git://"))
        .or_else(|| url.strip_prefix("ssh://"))
    {
        // https://github.com/org/repo.git → ("github.com", "org/repo.git")
        let mut parts = after_scheme.splitn(2, '/');
        let h = parts.next().unwrap_or("");
        // strip user@ prefix (ssh://git@github.com/...)
        let h = h.split('@').next_back().unwrap_or(h);
        let p = parts.next().unwrap_or("");
        (h.to_string(), p.to_string())
    } else {
        return String::new();
    };

    // For GitHub repos, show "Github: org/repo" instead of the domain
    if host == "github.com" {
        let slug = path.trim_end_matches(".git");
        if !slug.is_empty() {
            return format!("Github: {slug}");
        }
    }

    host
}

/// Build a set of source names that are external (git).
fn external_source_set(config: &crate::config::Config) -> std::collections::HashSet<String> {
    config
        .source
        .iter()
        .filter(|s| s.source_type == "git")
        .map(|s| s.name.clone())
        .collect()
}

/// Format a breakdown like "3 external, 2 local" or just "3 external" / "3 local".
fn source_breakdown(external: usize, local: usize) -> String {
    match (external, local) {
        (0, l) => format!("{} local", l),
        (e, 0) => format!("{} external", e),
        (e, l) => format!("{} external, {} local", e, l),
    }
}

/// Resolve a skill identifier (exact, glob, or freeform) to a list of (source_name, fully_qualified_id).
fn resolve_skills_for_bundle(
    skill_id: &str,
    registry: &crate::registry::Registry,
) -> anyhow::Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    if crate::registry::is_glob(skill_id) {
        let matches = registry.match_skills(skill_id);
        if matches.is_empty() {
            anyhow::bail!("no skills matched pattern '{}'", skill_id);
        }
        for (src, plugin, skill) in &matches {
            let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
            results.push((src.to_string(), fq));
        }
    } else {
        match registry.find_skill(skill_id) {
            Ok((src, plug, sk)) => {
                let fq = crate::output::plain_identity(src, plug, &sk.name);
                results.push((src.to_string(), fq));
            }
            Err(_) => {
                let matches = registry.match_skills(skill_id);
                if matches.is_empty() {
                    anyhow::bail!("no skills matched '{}'", skill_id);
                }
                for (src, plugin, skill) in &matches {
                    let fq = crate::output::plain_identity(src, &plugin.name, &skill.name);
                    results.push((src.to_string(), fq));
                }
            }
        }
    }
    Ok(results)
}

const AGENT_PREFIXES: &[(&str, &str)] = &[
    ("claude", ".claude"),
    ("codex", ".codex"),
    ("cursor", ".cursor"),
];

/// Scan home and cwd for agent installation directories.
/// Returns (agent_type, path) for each found candidate.
pub fn detect_agents() -> Vec<(String, std::path::PathBuf)> {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| dirs::home_dir().ok_or(()))
        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

    let cwd = std::env::current_dir().unwrap_or_default();
    let dirs_to_scan: Vec<&std::path::Path> = if cwd == home {
        vec![&home]
    } else {
        vec![&home, &cwd]
    };

    let mut candidates: Vec<(String, std::path::PathBuf)> = Vec::new();
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
                            candidates.push((agent.to_string(), path));
                        }
                    }
                }
            }
        }
    }
    candidates.sort_by(|a, b| a.1.cmp(&b.1));
    candidates
}

/// Add all detected agents to config (auto-add, no per-agent prompt).
/// Returns count of agents added.
pub fn add_detected_agents(config: &mut crate::config::Config, quiet: bool) -> usize {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| dirs::home_dir().ok_or(()))
        .unwrap_or_else(|_| std::path::PathBuf::from("~"));

    let candidates = detect_agents();
    let mut added = 0;
    for (agent, path) in &candidates {
        if config.agent.iter().any(|t| t.path == *path) {
            continue;
        }
        let agent_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(agent)
            .trim_start_matches('.')
            .to_string();
        let scope = if path.starts_with(&home) {
            "machine"
        } else {
            "repo"
        };
        let sync = if scope == "repo" { "explicit" } else { "auto" };
        config.agent.push(crate::config::AgentConfig {
            name: agent_name.clone(),
            agent_type: agent.to_string(),
            path: path.clone(),
            scope: scope.to_string(),
            sync: sync.to_string(),
        });
        if !quiet {
            use colored::Colorize;
            let display_path = if let Some(home_str) = home.to_str() {
                path.to_string_lossy().replacen(home_str, "~", 1)
            } else {
                path.to_string_lossy().to_string()
            };
            let skills_desc = match agent.as_str() {
                "cursor" => format!("(skills in {}/)", display_path),
                _ => format!("(skills in {}/skills/)", display_path),
            };
            println!(
                "  {} {}  {}  {}",
                "✓".green(),
                agent_name.bold(),
                display_path.dimmed(),
                skills_desc.dimmed(),
            );
        }
        added += 1;
    }
    added
}

fn resolve_agents<'a>(
    config: &'a crate::config::Config,
    agent_names: &Option<Vec<String>>,
    all_agents: bool,
) -> anyhow::Result<Vec<&'a crate::config::AgentConfig>> {
    if all_agents {
        if config.agent.is_empty() {
            anyhow::bail!("no agents configured. Use `loadout agent add` first.");
        }
        return Ok(config.agent.iter().collect());
    }

    if let Some(names) = agent_names {
        let mut agents = Vec::new();
        for name in names {
            let ac = config
                .agent
                .iter()
                .find(|ac| ac.name == *name)
                .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", name))?;
            agents.push(ac);
        }
        if agents.is_empty() {
            anyhow::bail!("no agents specified");
        }
        return Ok(agents);
    }

    // Default: auto-sync agents
    let auto: Vec<_> = config.agent.iter().filter(|t| t.sync == "auto").collect();
    if auto.is_empty() {
        anyhow::bail!("no agents configured. Use `loadout agent add` first.");
    }
    Ok(auto)
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init { url } => {
            let path = crate::config::config_path(cli.config.as_deref());
            if path.exists() {
                if url.is_some() && !cli.quiet {
                    println!(
                        "Config already exists at {}. Use `loadout add` instead.",
                        path.display()
                    );
                } else if !cli.quiet {
                    println!(
                        "Config already exists at {}. Use `loadout config edit` to modify.",
                        path.display()
                    );
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

            // Migrate legacy registry.json to .loadout/
            let legacy_registry = data.join("registry.json");
            let new_registry = crate::config::internal_dir().join("registry.json");
            if legacy_registry.exists() && !new_registry.exists() {
                std::fs::rename(&legacy_registry, &new_registry)?;
            }

            // Write .gitignore
            let gitignore_path = data.join(".gitignore");
            if !gitignore_path.exists() {
                std::fs::write(&gitignore_path, "external/\n.loadout/\n")?;
            }

            let default_config = crate::config::DEFAULT_CONFIG;
            std::fs::write(&path, default_config)?;
            if !cli.quiet {
                println!("Initialized loadout at {}", data.display());
            }

            // If URL provided, fetch into cache and register as source
            if let Some(ref url_str) = url {
                let source_url = crate::source::SourceUrl::parse(url_str)?;
                let source_name = source_url.default_name();
                let cache_path = crate::config::cache_dir().join(&source_name);

                crate::source::fetch::fetch(&source_url, &cache_path, None)?;

                let structure = crate::source::detect::detect(&cache_path)?;
                let registered =
                    crate::source::normalize::normalize(&source_name, &cache_path, &structure)?;

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

            // --- Interactive wizard steps ---

            // Step 1: git init the data dir
            let should_git_init = if data.join(".git").exists() {
                false // already a git repo
            } else if cli.quiet || !crate::prompt::is_interactive() {
                true // non-interactive default: yes
            } else {
                crate::prompt::confirm_or_override(
                    "Initialize git in loadout data dir? [Y/n]",
                    "Y",
                    cli.quiet,
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
                        if !cli.quiet {
                            println!("Initialized git in {}", data.display());
                        }
                    }
                    Ok(o) => {
                        if cli.verbose {
                            eprintln!(
                                "warning: git init failed: {}",
                                String::from_utf8_lossy(&o.stderr).trim()
                            );
                        }
                    }
                    Err(_) => {
                        if cli.verbose {
                            eprintln!("warning: git not found, skipping git init");
                        }
                    }
                }
            }

            // Step 2: detect and add agents
            let should_detect = if cli.quiet || !crate::prompt::is_interactive() {
                true
            } else {
                crate::prompt::confirm_or_override(
                    "Detect and add agents? [Y/n]",
                    "Y",
                    cli.quiet,
                )
                .to_uppercase()
                    != "N"
            };
            if should_detect {
                let mut config = crate::config::load(cli.config.as_deref())?;
                let added = add_detected_agents(&mut config, cli.quiet);
                if added > 0 {
                    crate::config::save(&config, cli.config.as_deref())?;
                } else if !cli.quiet {
                    println!("  No agents found");
                }
            }

            // Step 3: offer popular marketplaces (skip if URL was provided)
            if url.is_none() && crate::prompt::is_interactive() && !cli.quiet {
                let names: Vec<&str> = crate::marketplace::KNOWN_MARKETPLACES
                    .iter()
                    .map(|(name, _)| *name)
                    .collect();
                let defaults: Vec<bool> = vec![true; names.len()];
                let selected = crate::prompt::multi_select(
                    "Add popular skill sources?",
                    &names,
                    &defaults,
                    cli.quiet,
                );

                if !selected.is_empty() {
                    let mut config = crate::config::load(cli.config.as_deref())?;
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

                        if !cli.quiet {
                            println!("Adding '{}'...", name);
                        }
                        match crate::source::fetch::fetch(&source_url, &cache_path, None) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("warning: failed to fetch '{}': {}", name, e);
                                continue;
                            }
                        }
                        match crate::source::detect::detect(&cache_path) {
                            Ok(structure) => {
                                match crate::source::normalize::normalize(
                                    &source_name,
                                    &cache_path,
                                    &structure,
                                ) {
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
                                        if !cli.quiet {
                                            println!("  Added source '{}'", source_name);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("warning: failed to normalize '{}': {}", name, e)
                                    }
                                }
                            }
                            Err(e) => eprintln!("warning: failed to detect '{}': {}", name, e),
                        }
                    }

                    crate::registry::save_registry(&registry, &data_dir)?;
                    crate::config::save(&config, cli.config.as_deref())?;
                }
            }

            Ok(())
        }
        Command::Add {
            url,
            source,
            plugin,
            skill,
            name,
            r#ref,
            symlink,
            copy,
        } => {
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
                // Use tree ref from URL when no explicit --ref provided
                let effective_ref = r#ref.as_deref().or_else(|| source_url.tree_ref());
                if !cli.quiet {
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
                        // Prompt only when the inferred plugin differs from source
                        let default_plugin = match &structure {
                            SourceStructure::SingleFile { .. }
                            | SourceStructure::SingleSkillDir { .. } => {
                                // plugin = source_name, no point prompting
                                None
                            }
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
                                crate::prompt::confirm_or_override("Plugin name", dp, cli.quiet);
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
                                let confirmed = crate::prompt::confirm_or_override(
                                    "Skill name",
                                    skill_name,
                                    cli.quiet,
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
                                    cli.quiet,
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
                if !cli.quiet && !crate::prompt::is_interactive() {
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

            if !cli.quiet {
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

                    if cli.verbose {
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
        Command::List { patterns, external, fzf } => {
            let data_dir = crate::config::data_dir();
            let config_for_list = crate::config::load(cli.config.as_deref())?;
            let mut registry = crate::registry::load_registry(&data_dir)?;
            let renames = crate::registry::reconcile_with_config(
                &mut registry,
                &config_for_list.source,
                &data_dir,
            )?;
            if !renames.is_empty() {
                crate::registry::save_registry(&registry, &data_dir)?;
                if !cli.quiet {
                    for r in &renames {
                        eprintln!("source renamed: {}", r);
                    }
                }
            }

            if external {
                // List external sources in table format
                if cli.json {
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
                        crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    output.info("No sources configured. Use `loadout add` to add one.");
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

                let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
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
                                // No exact match — fall back to contains search
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
                    let output = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    output.info("No skills to browse.");
                    return Ok(());
                }

                // Build lines: "identity\tpath" — fzf shows identity, preview uses path
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

                let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                out.status(
                    "Identity",
                    &crate::output::format_identity(source_name, plugin_name, &skill.name),
                );
                out.status(
                    "Description",
                    skill.description.as_deref().unwrap_or("(none)"),
                );
                if cli.verbose {
                    out.status("Path", &skill.path.display().to_string());
                }
            } else if cli.json {
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
                        output.info("No skills found. Add a source with `loadout add`");
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
        Command::Status => {
            let config = crate::config::load(cli.config.as_deref())?;
            let data_dir = crate::config::data_dir();
            let mut registry = crate::registry::load_registry(&data_dir)?;
            let renames = crate::registry::reconcile_with_config(
                &mut registry,
                &config.source,
                &data_dir,
            )?;
            if !renames.is_empty() {
                crate::registry::save_registry(&registry, &data_dir)?;
                if !cli.quiet {
                    for r in &renames {
                        eprintln!("source renamed: {}", r);
                    }
                }
            }

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

            if cli.json {
                let json = serde_json::json!({
                    "sources": config.source.len(),
                    "agents": config.agent.len(),
                    "plugins": registry.sources.iter().flat_map(|s| &s.plugins).count(),
                    "skills": total_skills,
                    "installed": total_installed,
                    "kits": config.kit.len(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);

            // Sources section
            out.header("Sources");
            if config.source.is_empty() {
                out.info("  (none)");
            } else {
                for src in &config.source {
                    let skill_count: usize = registry
                        .sources
                        .iter()
                        .find(|rs| rs.name == src.name)
                        .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                        .unwrap_or(0);
                    let version = src.r#ref.as_deref().unwrap_or("latest");
                    let mode_str = src.mode.as_deref().unwrap_or("");
                    let detail = if mode_str.is_empty() {
                        format!("{} skills, @ {}", skill_count, version)
                    } else {
                        format!("{} skills, @ {}, {}", skill_count, version, mode_str)
                    };
                    println!(
                        "  {} {}",
                        src.name.bold(),
                        detail.dimmed(),
                    );
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
                        format!("{} installed, scope: {}, sync: {}", installed_count, ac.scope, ac.sync).dimmed(),
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
            out.status("Total", &format!(
                "{} sources, {} plugins, {} skills, {} installed",
                config.source.len(),
                registry.sources.iter().flat_map(|s| &s.plugins).count(),
                total_skills,
                total_installed,
            ));

            Ok(())
        }
        Command::Remove { name, force } => {
            let config_path_str = cli.config.as_deref();
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
                    crate::prompt::select_from("Select source to remove", &source_names, cli.quiet)?
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

            if !installed_on.is_empty() && !cli.quiet {
                eprintln!(
                    "warning: source '{}' has installed skills on: {}",
                    name,
                    installed_on.join(", ")
                );
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
        Command::Update {
            name,
            r#ref: update_ref,
        } => {
            let config_path_str = cli.config.as_deref();
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
                if !cli.quiet {
                    for r in &renames {
                        eprintln!("source renamed: {}", r);
                    }
                }
            }

            if update_ref.is_some() && name.is_none() {
                anyhow::bail!(
                    "--ref requires a source name (e.g., loadout update my-source --ref v2.0)"
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
                            if let Err(e) =
                                crate::source::fetch::fetch(&source_url, &cache_path, None)
                            {
                                errors.push(format!("{}: {}", src.name, e));
                                continue;
                            }
                        }
                        let _ = path; // used via source_url
                    }
                    crate::source::SourceUrl::Git(..) => {
                        if let Some(ref new_ref) = update_ref {
                            // Ref switch: fetch + checkout new ref + update config
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
                                // Unpin: fetch + reset to default branch
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
                                    // Pinned to a tag — warn and skip
                                    if !cli.quiet {
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
                        // Re-extract archive to cache
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

            if !cli.dry_run {
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
        Command::Kit {
            command: kit_cmd,
        } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;
            let data_dir = crate::config::data_dir();

            match kit_cmd {
                KitCommand::Create { name, skills } => {
                    let out =
                        crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
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
                            let resolved = resolve_skills_for_bundle(
                                skill_id, &registry,
                            )?;
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
                    let out =
                        crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    if !config.kit.contains_key(&name) {
                        anyhow::bail!("kit '{}' not found", name);
                    }

                    let execute = force && !cli.dry_run;
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
                    let kits: Vec<(&String, &crate::config::KitConfig)> =
                        if patterns.is_empty() {
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

                    if cli.json {
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

                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    if kits.is_empty() {
                        if patterns.is_empty() {
                            out.info("No kits configured. Use `loadout kit create` to create one.");
                        } else {
                            out.info("No kits matched the given pattern(s)");
                        }
                        return Ok(());
                    }

                    for (name, b) in &kits {
                        println!("{} {}", name.bold(), format!("({})", b.skills.len()).dimmed());
                        for (i, skill_id) in b.skills.iter().enumerate() {
                            let connector = if i == b.skills.len() - 1 { "└──" } else { "├──" };
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

                    if cli.json {
                        let json = serde_json::json!({
                            "name": name,
                            "skills": kit.skills,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
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
                    let out =
                        crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    if !config.kit.contains_key(&name) {
                        anyhow::bail!("kit '{}' not found", name);
                    }

                    let mut registry = crate::registry::load_registry(&data_dir)?;
                    let renames = crate::registry::reconcile_with_config(
                        &mut registry,
                        &config.source,
                        &data_dir,
                    )?;
                    if !renames.is_empty() {
                        crate::registry::save_registry(&registry, &data_dir)?;
                        if !cli.quiet {
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
                        let resolved = resolve_skills_for_bundle(
                            skill_id, &registry,
                        )?;
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
                        added, name, source_breakdown(external, local), total,
                    ));
                    Ok(())
                }
                KitCommand::Drop { name, skills } => {
                    let out =
                        crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
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
        Command::Agent {
            command: agent_cmd,
        } => {
            let config_path_str = cli.config.as_deref();
            let mut config = crate::config::load(config_path_str)?;

            // Known built-in agent types
            const KNOWN_AGENTS: &[&str] = &["claude", "codex", "cursor", "gemini", "vscode"];

            match agent_cmd {
                AgentCommand::Add {
                    agent,
                    path,
                    name,
                    scope,
                    sync,
                } => {
                    // Validate agent type against built-in + custom adapters
                    if !KNOWN_AGENTS.contains(&agent.as_str())
                        && !config.adapter.contains_key(&agent)
                    {
                        let available: Vec<String> = KNOWN_AGENTS
                            .iter()
                            .map(|s| s.to_string())
                            .chain(config.adapter.keys().cloned())
                            .collect();
                        anyhow::bail!(
                            "unknown agent type '{}'. Available: {}",
                            agent,
                            available.join(", ")
                        );
                    }

                    let agent_name = name.unwrap_or_else(|| format!("{}-{}", agent, scope));

                    // Check for duplicate name
                    if config.agent.iter().any(|t| t.name == agent_name) {
                        anyhow::bail!("agent '{}' already exists", agent_name);
                    }

                    // Resolve path: default based on agent + scope
                    let agent_path = if let Some(p) = path {
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
                        config.agent.push(crate::config::AgentConfig {
                            name: agent_name.clone(),
                            agent_type: agent.clone(),
                            path: agent_path,
                            scope,
                            sync: actual_sync,
                        });
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        if cli.dry_run {
                            println!("  (dry run) would add agent '{}'", agent_name);
                        } else {
                            println!("Added agent '{}'", agent_name);
                        }
                    }
                    Ok(())
                }
                AgentCommand::Remove { name, force } => {
                    if !config.agent.iter().any(|t| t.name == name) {
                        anyhow::bail!("agent '{}' not found", name);
                    }

                    let execute = force && !cli.dry_run;
                    if execute {
                        config.agent.retain(|t| t.name != name);
                        crate::config::save(&config, config_path_str)?;
                    }

                    if !cli.quiet {
                        if execute {
                            println!("Removed agent '{}' (installed skills preserved)", name);
                        } else {
                            println!("Would remove agent '{}'", name);
                            println!("Use --force to remove");
                        }
                    }
                    Ok(())
                }
                AgentCommand::List => {
                    if cli.json {
                        let entries: Vec<serde_json::Value> = config
                            .agent
                            .iter()
                            .map(|t| {
                                let adapter = crate::agent::resolve_adapter(t, &config.adapter).ok();
                                let installed_count = adapter
                                    .as_ref()
                                    .and_then(|a| a.installed_skills(&t.path).ok())
                                    .map(|v| v.len())
                                    .unwrap_or(0);
                                serde_json::json!({
                                    "name": t.name,
                                    "agent": t.agent_type,
                                    "path": t.path,
                                    "scope": t.scope,
                                    "sync": t.sync,
                                    "installed": installed_count,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    if config.agent.is_empty() {
                        out.info("No agents configured. Use `loadout agent add` to add one.");
                        return Ok(());
                    }

                    for ac in &config.agent {
                        let adapter = crate::agent::resolve_adapter(ac, &config.adapter).ok();
                        let installed = adapter
                            .as_ref()
                            .and_then(|a| a.installed_skills(&ac.path).ok())
                            .unwrap_or_default();

                        println!(
                            "{} {} {}",
                            ac.name.bold(),
                            format!("({})", ac.agent_type).cyan(),
                            format!("— {}", ac.path.display()).dimmed(),
                        );
                        println!(
                            "  {} {} {}",
                            "scope:".dimmed(),
                            ac.scope,
                            format!("  sync: {}  installed: {}", ac.sync, installed.len()).dimmed(),
                        );
                    }
                    Ok(())
                }
                AgentCommand::Show { name } => {
                    let agent_cfg = config
                        .agent
                        .iter()
                        .find(|t| t.name == name)
                        .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", name))?;

                    if cli.json {
                        let json = serde_json::json!({
                            "name": agent_cfg.name,
                            "type": agent_cfg.agent_type,
                            "path": agent_cfg.path,
                            "scope": agent_cfg.scope,
                            "sync": agent_cfg.sync,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        return Ok(());
                    }

                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                    out.status("Name", &agent_cfg.name);
                    out.status("Type", &agent_cfg.agent_type);
                    out.status("Path", &agent_cfg.path.display().to_string());
                    out.status("Scope", &agent_cfg.scope);
                    out.status("Sync", &agent_cfg.sync);

                    // List installed skills if the directory exists
                    let skills_dir = agent_cfg.path.join("skills");
                    if skills_dir.is_dir() {
                        let mut installed = Vec::new();
                        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                            for entry in entries.flatten() {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                                    && entry.path().join("SKILL.md").exists()
                                {
                                    installed.push(entry.file_name().to_string_lossy().to_string());
                                }
                            }
                        }
                        if !installed.is_empty() {
                            installed.sort();
                            out.status("Installed", &installed.len().to_string());
                            out.info("");
                            let tree: Vec<(usize, String)> =
                                installed.into_iter().map(|s| (0, s)).collect();
                            out.tree(&tree);
                        }
                    }

                    Ok(())
                }
                AgentCommand::Detect { force } => {
                    let candidates = detect_agents();

                    let mut found: Vec<(String, std::path::PathBuf, bool)> = Vec::new();
                    for (agent, path) in &candidates {
                        let already = config.agent.iter().any(|t| t.path == *path);
                        found.push((agent.clone(), path.clone(), already));
                    }

                    if cli.json {
                        let entries: Vec<serde_json::Value> = found
                            .iter()
                            .map(|(agent, path, registered)| {
                                serde_json::json!({
                                    "agent": agent,
                                    "path": path,
                                    "registered": registered,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                        return Ok(());
                    }

                    if found.is_empty() {
                        if !cli.quiet {
                            println!("No agent configurations found.");
                        }
                        return Ok(());
                    }

                    if force {
                        let added = add_detected_agents(&mut config, cli.quiet);
                        if added > 0 {
                            crate::config::save(&config, config_path_str)?;
                        }
                    } else {
                        let home = std::env::var("HOME")
                            .map(std::path::PathBuf::from)
                            .or_else(|_| dirs::home_dir().ok_or(()))
                            .unwrap_or_else(|_| std::path::PathBuf::from("~"));
                        let out =
                            crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);
                        let mut added = 0;
                        for (agent, path, registered) in &found {
                            if *registered {
                                out.info(&format!(
                                    "{} at {} (already registered)",
                                    agent,
                                    path.display()
                                ));
                                continue;
                            }
                            let agent_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(agent)
                                .trim_start_matches('.')
                                .to_string();
                            eprint!(
                                "Add {} at {} as agent '{}'? [y/N] ",
                                agent,
                                path.display(),
                                agent_name
                            );
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap_or(0);
                            if input.trim().eq_ignore_ascii_case("y") {
                                let scope = if path.starts_with(&home) {
                                    "machine"
                                } else {
                                    "repo"
                                };
                                let sync = if scope == "repo" { "explicit" } else { "auto" };
                                config.agent.push(crate::config::AgentConfig {
                                    name: agent_name.clone(),
                                    agent_type: agent.to_string(),
                                    path: path.clone(),
                                    scope: scope.to_string(),
                                    sync: sync.to_string(),
                                });
                                out.success(&format!("Added agent '{}'", agent_name));
                                added += 1;
                            }
                        }
                        if added > 0 {
                            crate::config::save(&config, config_path_str)?;
                        }
                    }

                    Ok(())
                }
                AgentCommand::Equip {
                    patterns,
                    agent,
                    all,
                    kit,
                    save,
                    force,
                    interactive,
                } => {
                    if patterns.is_empty() && kit.is_none() {
                        eprintln!("error: equip requires skill patterns or --kit");
                        std::process::exit(2);
                    }

                    let data_dir = crate::config::data_dir();
                    let mut registry = crate::registry::load_registry(&data_dir)?;
                    let renames = crate::registry::reconcile_with_config(
                        &mut registry,
                        &config.source,
                        &data_dir,
                    )?;
                    if !renames.is_empty() {
                        crate::registry::save_registry(&registry, &data_dir)?;
                        if !cli.quiet {
                            for r in &renames {
                                eprintln!("source renamed: {}", r);
                            }
                        }
                    }

                    let agents = resolve_agents(&config, &agent, all)?;

                    // Collect skills to equip with provenance: (source, plugin, skill)
                    let mut skills_to_apply: Vec<(&str, &str, &crate::registry::RegisteredSkill)> =
                        Vec::new();

                    // From positional patterns
                    for pattern in &patterns {
                        if crate::registry::is_glob(pattern) {
                            let matches = registry.match_skills(pattern);
                            if matches.is_empty() {
                                anyhow::bail!("no skills matched pattern '{}'", pattern);
                            }
                            for (src, plugin, s) in matches {
                                skills_to_apply.push((src, &plugin.name, s));
                            }
                        } else {
                            match registry.find_skill(pattern) {
                                Ok((src, plug, s)) => {
                                    skills_to_apply.push((src, plug, s));
                                }
                                Err(_) => {
                                    // Fall back to glob-style match
                                    let matches = registry.match_skills(pattern);
                                    if matches.is_empty() {
                                        anyhow::bail!("no skills matched '{}'", pattern);
                                    }
                                    for (src, plugin, s) in matches {
                                        skills_to_apply.push((src, &plugin.name, s));
                                    }
                                }
                            }
                        }
                    }

                    // From --kit
                    if let Some(ref kit_name) = kit {
                        let kit_cfg = config
                            .kit
                            .get(kit_name)
                            .ok_or_else(|| anyhow::anyhow!("kit '{}' not found", kit_name))?;
                        for skill_id in &kit_cfg.skills {
                            let (src, plug, s) = registry.find_skill(skill_id)?;
                            skills_to_apply.push((src, plug, s));
                        }
                    }

                    // Interactive confirmation (default when not --force)
                    if !force && !cli.dry_run && !skills_to_apply.is_empty() && crate::prompt::is_interactive() {
                        eprintln!("Skills to equip:");
                        for (src, plug, s) in &skills_to_apply {
                            eprintln!("  {}", crate::output::format_identity(src, plug, &s.name));
                        }
                        eprintln!("Agents:");
                        for ac in &agents {
                            eprintln!("  {}", ac.name.bold());
                        }
                        eprint!("Proceed? [y/N] ");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap_or(0);
                        if !input.trim().eq_ignore_ascii_case("y") {
                            eprintln!("Aborted.");
                            return Ok(());
                        }
                    }

                    // Apply to each agent with conflict detection
                    let mut new_count: usize = 0;
                    let mut updated_count: usize = 0;
                    let mut unchanged_count: usize = 0;
                    let mut conflict_skipped: usize = 0;
                    let mut force_remaining = force;
                    let mut reg = registry.clone();

                    for ac in &agents {
                        let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;

                        // Detect conflicts when not interactive and not forced
                        if !force && !interactive && !cli.dry_run {
                            let mut conflicts = Vec::new();
                            for (_, _, s) in &skills_to_apply {
                                let status = adapter.compare_skill(s, &ac.path)?;
                                if status == crate::agent::SkillStatus::Changed {
                                    conflicts.push(s.name.clone());
                                }
                            }
                            if !conflicts.is_empty() {
                                eprintln!(
                                    "error: {} skill(s) have changed at agent '{}':",
                                    conflicts.len(),
                                    ac.name
                                );
                                for name in &conflicts {
                                    eprintln!("  - {}", name);
                                }
                                eprintln!();
                                eprintln!("Use --force to overwrite, or -i for interactive resolution.");
                                std::process::exit(1);
                            }
                        }

                        for (src_name, plug_name, s) in &skills_to_apply {
                            let status = adapter.compare_skill(s, &ac.path)?;

                            if cli.dry_run {
                                if !cli.quiet {
                                    let label = match status {
                                        crate::agent::SkillStatus::New => "new",
                                        crate::agent::SkillStatus::Unchanged => "unchanged",
                                        crate::agent::SkillStatus::Changed => "changed",
                                    };
                                    println!(
                                        "  (dry run) {} → {} [{}]",
                                        crate::output::format_identity(src_name, plug_name, &s.name),
                                        ac.name,
                                        label
                                    );
                                }
                                continue;
                            }

                            match status {
                                crate::agent::SkillStatus::Unchanged => {
                                    unchanged_count += 1;
                                    continue;
                                }
                                crate::agent::SkillStatus::New => {
                                    adapter.install_skill(s, &ac.path)?;
                                    record_provenance(&mut reg, &data_dir, ac, src_name, plug_name, s);
                                    new_count += 1;
                                }
                                crate::agent::SkillStatus::Changed => {
                                    if force_remaining {
                                        adapter.install_skill(s, &ac.path)?;
                                        record_provenance(&mut reg, &data_dir, ac, src_name, plug_name, s);
                                        updated_count += 1;
                                    } else if interactive {
                                        let action = prompt_conflict(s, &adapter, &ac.path)?;
                                        match action {
                                            ConflictAction::Skip => {
                                                conflict_skipped += 1;
                                            }
                                            ConflictAction::Overwrite => {
                                                adapter.install_skill(s, &ac.path)?;
                                                record_provenance(
                                                    &mut reg, &data_dir, ac, src_name, plug_name, s,
                                                );
                                                updated_count += 1;
                                            }
                                            ConflictAction::ForceAll => {
                                                adapter.install_skill(s, &ac.path)?;
                                                record_provenance(
                                                    &mut reg, &data_dir, ac, src_name, plug_name, s,
                                                );
                                                updated_count += 1;
                                                force_remaining = true;
                                            }
                                            ConflictAction::Quit => {
                                                crate::registry::save_registry(&reg, &data_dir)?;
                                                print_apply_summary(
                                                    new_count,
                                                    updated_count,
                                                    unchanged_count,
                                                    conflict_skipped,
                                                    cli.quiet,
                                                );
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !cli.dry_run {
                        crate::registry::save_registry(&reg, &data_dir)?;
                    }

                    // --save: persist the resolved skill set under the --kit name
                    if save {
                        let Some(ref kit_name) = kit else {
                            anyhow::bail!("--save requires --kit to specify the kit name");
                        };
                        let mut config = crate::config::load(cli.config.as_deref())?;
                        let mut skill_ids: Vec<String> = Vec::new();
                        for (src, plug, s) in &skills_to_apply {
                            let fq = crate::output::plain_identity(src, plug, &s.name);
                            if !skill_ids.contains(&fq) {
                                skill_ids.push(fq);
                            }
                        }
                        config.kit.insert(
                            kit_name.clone(),
                            crate::config::KitConfig { skills: skill_ids },
                        );
                        crate::config::save(&config, cli.config.as_deref())?;
                        if !cli.quiet {
                            println!("Saved kit '{}'", kit_name);
                        }
                    }

                    if !cli.quiet && !cli.dry_run {
                        print_apply_summary(
                            new_count,
                            updated_count,
                            unchanged_count,
                            conflict_skipped,
                            cli.quiet,
                        );
                    }
                    Ok(())
                }
                AgentCommand::Unequip {
                    patterns,
                    agent,
                    all,
                    kit,
                    force,
                } => {
                    if patterns.is_empty() && kit.is_none() {
                        eprintln!("error: unequip requires skill patterns or --kit");
                        std::process::exit(2);
                    }

                    let data_dir = crate::config::data_dir();
                    let mut registry = crate::registry::load_registry(&data_dir)?;
                    let renames = crate::registry::reconcile_with_config(
                        &mut registry,
                        &config.source,
                        &data_dir,
                    )?;
                    if !renames.is_empty() {
                        crate::registry::save_registry(&registry, &data_dir)?;
                        if !cli.quiet {
                            for r in &renames {
                                eprintln!("source renamed: {}", r);
                            }
                        }
                    }
                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);

                    let agents = resolve_agents(&config, &agent, all)?;

                    // Collect skill names to remove
                    let mut skill_names: Vec<String> = Vec::new();

                    for pattern in &patterns {
                        if crate::registry::is_glob(pattern) {
                            let matches = registry.match_skills(pattern);
                            if matches.is_empty() {
                                anyhow::bail!("no skills matched pattern '{}'", pattern);
                            }
                            for (_, _, s) in matches {
                                if !skill_names.contains(&s.name) {
                                    skill_names.push(s.name.clone());
                                }
                            }
                        } else {
                            match registry.find_skill(pattern) {
                                Ok((_, _, s)) => {
                                    if !skill_names.contains(&s.name) {
                                        skill_names.push(s.name.clone());
                                    }
                                }
                                Err(_) => {
                                    let matches = registry.match_skills(pattern);
                                    if matches.is_empty() {
                                        anyhow::bail!("no skills matched '{}'", pattern);
                                    }
                                    for (_, _, s) in matches {
                                        if !skill_names.contains(&s.name) {
                                            skill_names.push(s.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(ref kit_name) = kit {
                        let kit_cfg = config
                            .kit
                            .get(kit_name)
                            .ok_or_else(|| anyhow::anyhow!("kit '{}' not found", kit_name))?;
                        for skill_id in &kit_cfg.skills {
                            let (_, _, s) = registry.find_skill(skill_id)?;
                            if !skill_names.contains(&s.name) {
                                skill_names.push(s.name.clone());
                            }
                        }
                    }

                    let execute = force && !cli.dry_run;
                    let mut total_removed = 0usize;
                    let mut _total_skipped = 0usize;

                    for ac in &agents {
                        let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;
                        let installed = adapter.installed_skills(&ac.path).unwrap_or_default();

                        for name in &skill_names {
                            if installed.contains(name) {
                                let identity = registry
                                    .installed
                                    .get(&ac.name)
                                    .and_then(|m| m.get(name))
                                    .map(|info| {
                                        crate::output::format_identity(&info.source, &info.plugin, &info.skill)
                                    })
                                    .unwrap_or_else(|| name.clone());

                                if execute {
                                    adapter.uninstall_skill(name, &ac.path)?;
                                    if let Some(agent_map) = registry.installed.get_mut(&ac.name) {
                                        agent_map.remove(name);
                                    }
                                    out.success(&format!(
                                        "Removed {} from {}",
                                        identity,
                                        ac.name.bold()
                                    ));
                                    total_removed += 1;
                                } else {
                                    out.info(&format!("  {} from {}", identity, ac.name.bold()));
                                    total_removed += 1;
                                }
                            } else {
                                _total_skipped += 1;
                            }
                        }
                    }

                    if !execute && total_removed > 0 {
                        out.info("");
                        out.warn("Preview only. Use --force to execute.");
                    }

                    if execute {
                        crate::registry::save_registry(&registry, &data_dir)?;
                        if !cli.quiet {
                            out.info(&format!(
                                "Removed {} skill(s) from {} agent(s)",
                                total_removed,
                                agents.len()
                            ));
                        }
                    } else if total_removed == 0 && !cli.quiet {
                        out.info("No matching skills found on agent(s).");
                    }

                    Ok(())
                }
                AgentCommand::Collect {
                    agent,
                    skill,
                    adopt,
                    force,
                } => {
                    let data_dir = crate::config::data_dir();
                    let mut registry = crate::registry::load_registry(&data_dir)?;
                    let renames = crate::registry::reconcile_with_config(
                        &mut registry,
                        &config.source,
                        &data_dir,
                    )?;
                    if !renames.is_empty() {
                        crate::registry::save_registry(&registry, &data_dir)?;
                        if !cli.quiet {
                            for r in &renames {
                                eprintln!("source renamed: {}", r);
                            }
                        }
                    }
                    let out = crate::output::Output::from_flags(cli.json, cli.quiet, cli.verbose);

                    let ac = config
                        .agent
                        .iter()
                        .find(|a| a.name == agent)
                        .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", agent))?;
                    let adapter = crate::agent::resolve_adapter(ac, &config.adapter)?;
                    let installed_on_agent = adapter.installed_skills(&ac.path)?;

                    if let Some(ref skill_name) = skill {
                        let agent_skill_dir = ac.path.join("skills").join(skill_name);
                        if !agent_skill_dir.exists() {
                            anyhow::bail!("skill '{}' not found on agent '{}'", skill_name, agent);
                        }

                        if adopt {
                            let provenance = registry
                                .installed
                                .get(&agent)
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
                            copy_dir_all(&agent_skill_dir, &dest_skill)?;

                            let plugin_json_dir = dest_plugin.join(".claude-plugin");
                            let plugin_json = plugin_json_dir.join("plugin.json");
                            if !plugin_json.exists() {
                                std::fs::create_dir_all(&plugin_json_dir)?;
                                let json = serde_json::json!({"name": plugin_name});
                                std::fs::write(&plugin_json, serde_json::to_string_pretty(&json)?)?;
                            }

                            generate_marketplace(&data_dir)?;
                            let identity =
                                crate::output::format_identity(&source_name, &plugin_name, skill_name);
                            out.success(&format!("Adopted {}", identity));
                        } else {
                            let provenance = registry
                                .installed
                                .get(&agent)
                                .and_then(|m| m.get(skill_name));

                            if let Some(info) = provenance {
                                let dest = data_dir.join(&info.origin);
                                std::fs::create_dir_all(&dest)?;
                                copy_dir_all(&agent_skill_dir, &dest)?;
                                let identity =
                                    crate::output::format_identity(&info.source, &info.plugin, &info.skill);
                                out.success(&format!("Collected {} → {}", identity, info.origin));
                            } else {
                                out.warn(&format!(
                                    "'{}' has no provenance. Use --adopt to claim it.",
                                    skill_name
                                ));
                            }
                        }
                    } else {
                        let agent_installs = registry.installed.get(&agent).cloned().unwrap_or_default();

                        let mut tracked = Vec::new();
                        let mut untracked = Vec::new();

                        for skill_name in &installed_on_agent {
                            if let Some(info) = agent_installs.get(skill_name) {
                                tracked.push((skill_name.clone(), info.clone()));
                            } else {
                                untracked.push(skill_name.clone());
                            }
                        }

                        if !tracked.is_empty() {
                            out.info("Tracked:");
                            for (_name, info) in &tracked {
                                let identity =
                                    crate::output::format_identity(&info.source, &info.plugin, &info.skill);
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
                                eprint!(
                                    "Adopt {} untracked skill(s) into plugins/local? [y/N] ",
                                    untracked.len()
                                );
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input).unwrap_or(0);
                                input.trim().eq_ignore_ascii_case("y")
                            } else {
                                false
                            };

                            if should_adopt {
                                let local_plugin = crate::config::plugins_dir().join("local");
                                for name in &untracked {
                                    let agent_skill_dir = ac.path.join("skills").join(name);
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
                        }

                        if tracked.is_empty() && untracked.is_empty() {
                            out.info("No skills found on agent.");
                        }
                    }

                    crate::registry::save_registry(&registry, &data_dir)?;
                    Ok(())
                }
            }
        }
        Command::Config {
            command: config_cmd,
        } => {
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
                        println!("Agents: {}", config.agent.len());
                        for t in &config.agent {
                            println!("  {} ({} @ {})", t.name, t.agent_type, t.path.display());
                        }
                        println!("Adapters: {}", config.adapter.len());
                        for name in config.adapter.keys() {
                            println!("  {}", name);
                        }
                        println!("Kits: {}", config.kit.len());
                        for (name, b) in &config.kit {
                            println!("  {} ({} skills)", name, b.skills.len());
                        }
                    }
                    Ok(())
                }
                ConfigCommand::Edit => {
                    let path = crate::config::config_path(cli.config.as_deref());
                    if !path.exists() {
                        anyhow::bail!("No config found. Run `loadout init` first.");
                    }
                    let editor = std::env::var("EDITOR")
                        .or_else(|_| std::env::var("VISUAL"))
                        .unwrap_or_else(|_| "vi".to_string());
                    let status = std::process::Command::new(&editor).arg(&path).status()?;
                    if !status.success() {
                        anyhow::bail!("editor exited with {}", status);
                    }
                    Ok(())
                }
            }
        }
        Command::Completions { shell, install } => {
            if install {
                match shell {
                    CompletionShell::Zsh => {
                        crate::completions::install_zsh(cli.quiet)?;
                    }
                    CompletionShell::Bash => {
                        crate::completions::install_bash(cli.quiet)?;
                    }
                    CompletionShell::Fish => {
                        crate::completions::install_fish(cli.quiet)?;
                    }
                }
            } else {
                let script = match shell {
                    CompletionShell::Zsh => crate::completions::ZSH_SCRIPT,
                    CompletionShell::Bash => crate::completions::BASH_SCRIPT,
                    CompletionShell::Fish => crate::completions::FISH_SCRIPT,
                };
                print!("{}", script);
            }
            Ok(())
        }
        Command::Complete { kind } => {
            let config = crate::config::load(cli.config.as_deref())?;
            let data_dir = crate::config::data_dir();
            let registry = crate::registry::load_registry(&data_dir)?;

            match kind.as_str() {
                "sources" => {
                    for s in &config.source {
                        println!("{}", s.name);
                    }
                }
                "plugins" => {
                    for src in &registry.sources {
                        for p in &src.plugins {
                            println!("{}:{}", src.name, p.name);
                        }
                    }
                }
                "skills" => {
                    for src in &registry.sources {
                        for p in &src.plugins {
                            for s in &p.skills {
                                println!("{}:{}/{}", src.name, p.name, s.name);
                            }
                        }
                    }
                }
                "agents" => {
                    for t in &config.agent {
                        println!("{}", t.name);
                    }
                }
                "kits" => {
                    for name in config.kit.keys() {
                        println!("{}", name);
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }
}

/// Recursively copy a directory's contents.
/// Record provenance for an applied skill.
fn record_provenance(
    reg: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    ac: &crate::config::AgentConfig,
    src_name: &str,
    plug_name: &str,
    s: &crate::registry::RegisteredSkill,
) {
    let origin = s
        .path
        .strip_prefix(data_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| s.path.display().to_string());
    let agent_map = reg.installed.entry(ac.name.clone()).or_default();
    agent_map.insert(
        s.name.clone(),
        crate::registry::InstalledSkill {
            source: src_name.to_string(),
            plugin: plug_name.to_string(),
            skill: s.name.clone(),
            origin,
        },
    );
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
    adapter: &crate::agent::Adapter,
    agent_path: &std::path::Path,
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
                show_skill_diff(skill, adapter, agent_path)?;
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
    adapter: &crate::agent::Adapter,
    agent_path: &std::path::Path,
) -> anyhow::Result<()> {
    let pairs = adapter.skill_file_pairs(skill, agent_path)?;

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
        for hunk in diff
            .unified_diff()
            .header("installed", "source")
            .iter_hunks()
        {
            eprint!("    {}", hunk);
        }
    }

    Ok(())
}

/// Print the apply summary line.
fn print_apply_summary(
    new_count: usize,
    updated_count: usize,
    unchanged_count: usize,
    conflict_skipped: usize,
    quiet: bool,
) {
    if quiet {
        return;
    }
    let applied = new_count + updated_count;
    let mut msg = format!(
        "Applied {} skill(s) ({} new, {} updated), skipped {} unchanged.",
        applied, new_count, updated_count, unchanged_count
    );
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
        "name": "loadout-marketplace",
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
