use crate::cli::flags::Flags;
use crate::cli::ConfigCommand;

pub(crate) fn run(command: ConfigCommand, flags: &Flags) -> anyhow::Result<()> {
    let config = crate::config::load(flags.config_path())?;
    match command {
        ConfigCommand::Show => {
            if flags.json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                let path = crate::config::config_path(flags.config_path());
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
            let path = crate::config::config_path(flags.config_path());
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
