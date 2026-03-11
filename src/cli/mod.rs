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
            eprintln!("skittle: init not yet implemented");
            std::process::exit(1);
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
        Command::Source { command: _ } => {
            eprintln!("skittle: source not yet implemented");
            std::process::exit(1);
        }
        Command::Plugin { command: _ } => {
            eprintln!("skittle: plugin not yet implemented");
            std::process::exit(1);
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
        Command::Config { command: _ } => {
            eprintln!("skittle: config not yet implemented");
            std::process::exit(1);
        }
        Command::Cache { command: _ } => {
            eprintln!("skittle: cache not yet implemented");
            std::process::exit(1);
        }
    }
}
