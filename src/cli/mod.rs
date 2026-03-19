pub mod args;
mod commands;
mod flags;
mod helpers;

pub use helpers::{add_detected_agents, detect_agents};

use clap::{Parser, Subcommand, ValueEnum};

const HELP_TEMPLATE: &str = "\
equip — agent skill manager

\x1b[1;4mUsage\x1b[0m
  equip @<agent> <patterns...>              equip skills to an agent
  equip @<agent> +<kit>                     equip a kit to an agent
  equip <command> [args...]                 manage sources, kits, agents

\x1b[1;4mShorthand\x1b[0m
  @<agent>           target agent(s)              expands to --agent <name>
  +<kit>             apply a kit                  expands to --kit <name>

\x1b[1;4mEquip Flags\x1b[0m
  -s, --save         save matched skills as the named kit
  -r, --remove       unequip instead of equip
  -f, --force        overwrite changed skills without prompting
  -i, --interactive  interactively resolve conflicts

\x1b[1;4mPatterns\x1b[0m
  Globs matched against skill identities (source:plugin/skill):
    \"dev*\"              all skills starting with dev
    \"legal/*\"           all skills in the legal plugin
    \"my-src:*\"          everything from a specific source

\x1b[1;4mCommands\x1b[0m
{subcommands}

\x1b[1;4mGlobal Options\x1b[0m
{options}

\x1b[1;4mExamples\x1b[0m
  equip init github.com/myorg/my-skills
  equip add github.com/anthropics/skills
  equip add github.com/anthropics/claude-plugins-official
  equip source update

  equip @claude \"dev*\"                     equip all dev skills to claude
  equip @claude +frontend-dev              equip a kit
  equip @claude +frontend-dev -s \"dev*\"    equip skills and save as kit
  equip @claude -r \"legal/*\"               unequip legal skills
";

#[derive(Parser)]
#[command(
    name = "equip",
    about = "equip — agent skill manager",
    version = env!("GIT_VERSION"),
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true,
    help_template = HELP_TEMPLATE,
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
    /// Initialize equip configuration
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

    /// Remove local skill(s), or remove a source by exact name
    Remove {
        /// Local skill identity/glob, or exact source name
        patterns: Vec<String>,

        /// Force removal without prompting
        #[arg(long)]
        force: bool,
    },

    /// Show current status
    Status,

    /// Collect skills from an agent back to the library
    Collect {
        /// Agent to collect from
        #[arg(long, value_name = "AGENT")]
        agent: String,

        /// Skill patterns (glob supported, e.g. "dev*", "legal/*")
        patterns: Vec<String>,

        /// Kit name
        #[arg(short, long)]
        kit: Option<String>,

        /// Link one untracked installed copy to an existing canonical skill identity
        #[arg(long, value_name = "IDENTITY")]
        link: Option<String>,

        /// Adopt untracked skills into the local source
        #[arg(long = "adopt-local", alias = "adopt")]
        adopt_local: bool,

        /// Auto-adopt all without prompting
        #[arg(short, long)]
        force: bool,

        /// Interactive skill selection
        #[arg(short, long)]
        interactive: bool,
    },

    /// Reindex sources from disk and reconcile registry paths
    Reconcile {
        /// Reconcile only one source name (use `local` for the local source)
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
    },

    /// Manage sources
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },

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

    /// Internal command — accessed via @agent/+kit shorthand syntax
    #[command(name = "_equip", hide = true)]
    Equip {
        /// Glob patterns matching skills
        patterns: Vec<String>,

        /// Agent name(s) to target
        #[arg(short, long, num_args = 1..)]
        agent: Option<Vec<String>>,

        /// Target all configured agents
        #[arg(long, conflicts_with = "agent")]
        all: bool,

        /// Kit name
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

        /// Remove instead of equip
        #[arg(short, long)]
        remove: bool,
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
pub enum SourceCommand {
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

    /// List configured sources
    List,

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
    List {
        /// Show installed skills for each agent
        #[arg(long)]
        show_skills: bool,

        /// Show fully installed kits for each agent
        #[arg(long)]
        show_kits: bool,
    },
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

    /// Collect skills from an agent back to source
    Collect {
        /// Agent to collect from
        #[arg(long, value_name = "AGENT")]
        agent: String,

        /// Skill patterns (glob supported, e.g. "dev*", "legal/*")
        patterns: Vec<String>,

        /// Kit name
        #[arg(short, long)]
        kit: Option<String>,

        /// Link one untracked installed copy to an existing canonical skill identity
        #[arg(long, value_name = "IDENTITY")]
        link: Option<String>,

        /// Adopt untracked skills into the local source
        #[arg(long = "adopt-local", alias = "adopt")]
        adopt_local: bool,

        /// Auto-adopt all without prompting
        #[arg(short, long)]
        force: bool,

        /// Interactive skill selection
        #[arg(short, long)]
        interactive: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Open config in editor
    Edit,
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let flags = flags::Flags::from_cli(&cli);
    match cli.command {
        Command::Init { url } => commands::init::run(url, &flags),
        Command::Add {
            url,
            source,
            plugin,
            skill,
            name,
            r#ref,
            symlink,
            copy,
        } => commands::source::run_add(
            url, source, plugin, skill, name, r#ref, symlink, copy, &flags,
        ),
        Command::List {
            patterns,
            external,
            fzf,
        } => commands::source::run_list(patterns, external, fzf, &flags),
        Command::Remove { patterns, force } => {
            commands::source::run_remove(patterns, force, &flags)
        }
        Command::Status => commands::status::run(&flags),
        Command::Collect {
            agent,
            patterns,
            kit,
            link,
            adopt_local,
            force,
            interactive,
        } => commands::collect::run(
            agent,
            patterns,
            kit,
            link,
            adopt_local,
            force,
            interactive,
            &flags,
        ),
        Command::Reconcile { source } => commands::source::run_reconcile(source, &flags),
        Command::Source { command } => commands::source::run(command, &flags),
        Command::Kit { command } => commands::kit::run(command, &flags),
        Command::Agent { command } => commands::agent::run(command, &flags),
        Command::Config { command } => commands::config::run(command, &flags),
        Command::Completions { shell, install } => {
            commands::completions::run(shell, install, &flags)
        }
        Command::Complete { kind } => commands::completions::run_complete(kind, &flags),
        Command::Equip {
            patterns,
            agent,
            all,
            kit,
            save,
            force,
            interactive,
            remove,
        } => commands::equip::run(
            patterns,
            agent,
            all,
            kit,
            save,
            force,
            interactive,
            remove,
            &flags,
        ),
    }
}
