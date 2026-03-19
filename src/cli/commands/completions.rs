use crate::cli::flags::Flags;
use crate::cli::CompletionShell;

pub(crate) fn run(shell: CompletionShell, install: bool, flags: &Flags) -> anyhow::Result<()> {
    if install {
        match shell {
            CompletionShell::Zsh => {
                crate::completions::install_zsh(flags.quiet)?;
            }
            CompletionShell::Bash => {
                crate::completions::install_bash(flags.quiet)?;
            }
            CompletionShell::Fish => {
                crate::completions::install_fish(flags.quiet)?;
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

pub(crate) fn run_complete(kind: String, flags: &Flags) -> anyhow::Result<()> {
    let config = crate::config::load(flags.config_path())?;
    let data_dir = crate::config::data_dir();
    let registry = crate::cli::helpers::load_effective_registry(&config, &data_dir, flags.quiet)?;

    match kind.as_str() {
        "sources" => {
            for s in &config.source {
                println!("{}", s.id);
            }
            if registry.sources.iter().any(|source| source.id == "local") {
                println!("local");
            }
        }
        "plugins" => {
            for src in &registry.sources {
                for p in &src.plugins {
                    println!("{}:{}", src.id, p.name);
                }
            }
        }
        "skills" => {
            for src in &registry.sources {
                for p in &src.plugins {
                    for s in &p.skills {
                        println!("{}:{}/{}", src.id, p.name, s.name);
                    }
                }
            }
        }
        "agents" => {
            for t in &config.agent {
                println!("{}", t.id);
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
