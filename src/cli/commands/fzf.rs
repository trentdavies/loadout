use anyhow::Result;

use crate::cli::flags::Flags;
use crate::cli::helpers::{load_context, resolve_skill_patterns};
use crate::fzf;

pub(crate) struct FzfArgs {
    pub patterns: Vec<String>,
    pub agent: Option<Vec<String>>,
    pub action: String,
    pub multi: bool,
}

pub(crate) fn run(args: FzfArgs, flags: &Flags) -> Result<()> {
    let FzfArgs {
        patterns,
        agent,
        action,
        multi,
    } = args;

    let ctx = load_context(flags)?;

    // Determine fzf input: stdin pipe or registry skills
    let items = if let Some(stdin_items) = fzf::read_stdin_items() {
        // Piped input — use as-is, optionally filtered by patterns
        if patterns.is_empty() {
            stdin_items
        } else {
            stdin_items
                .into_iter()
                .filter(|item| {
                    patterns.iter().any(|pat| {
                        glob_match::glob_match(pat, &item.display) || item.display.contains(pat)
                    })
                })
                .collect()
        }
    } else {
        // Interactive — pull from registry
        let skills = if patterns.is_empty() {
            ctx.registry.all_skills()
        } else {
            resolve_skill_patterns(&patterns, &ctx.registry, true)?
        };

        if skills.is_empty() {
            let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            output.info("No skills found.");
            return Ok(());
        }

        fzf::skills_to_fzf_items(&skills)
    };

    if items.is_empty() {
        let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        output.info("No skills to browse.");
        return Ok(());
    }

    let opts = fzf::skill_browse_options(multi);

    let selected = fzf::run_fzf(&items, &opts)?;

    if selected.is_empty() {
        return Ok(());
    }

    match action.as_str() {
        "print" => {
            for item in &selected {
                println!("{}", item.display);
            }
        }
        "equip" => {
            let agent_names = agent.unwrap_or_default();
            if agent_names.is_empty() {
                anyhow::bail!(
                    "equip action requires --agent. Usage: equip fzf --action equip --agent claude"
                );
            }

            let identities: Vec<String> = selected.iter().map(|s| s.display.clone()).collect();

            crate::cli::commands::equip::run(
                crate::cli::commands::equip::EquipArgs {
                    patterns: identities,
                    agent: Some(agent_names),
                    all: false,
                    kit: None,
                    save: false,
                    force: false,
                    interactive: false,
                    remove: false,
                    fzf: false,
                },
                flags,
            )?;
        }
        "remove" => {
            let identities: Vec<String> = selected.iter().map(|s| s.display.clone()).collect();
            crate::cli::commands::source::run_remove(identities, false, flags)?;
        }
        other => {
            anyhow::bail!(
                "unknown action '{}'. Valid actions: print, equip, remove",
                other
            );
        }
    }

    Ok(())
}
