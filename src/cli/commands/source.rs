use colored::Colorize;

use crate::cli::flags::Flags;
use crate::cli::helpers::{
    extract_domain, load_context, resolve_skill_patterns, skill_list_status, SkillListStatus,
};

enum AddedSourceSummary {
    External { source_name: String },
    Local(crate::source::LocalImport),
}

struct StagedParsedSource {
    parsed: crate::source::ParsedSource,
    _temp_dir: Option<tempfile::TempDir>,
}

pub(crate) fn run(command: crate::cli::SourceCommand, flags: &Flags) -> anyhow::Result<()> {
    match command {
        crate::cli::SourceCommand::Add {
            url,
            source,
            plugin,
            skill,
            name,
            r#ref,
            symlink,
            copy,
        } => run_add(
            AddArgs {
                url,
                source,
                plugin,
                skill,
                name,
                r#ref,
                symlink,
                copy,
            },
            flags,
        ),
        crate::cli::SourceCommand::List => run_source_list(flags),
        crate::cli::SourceCommand::Remove { name, force } => run_source_remove(name, force, flags),
        crate::cli::SourceCommand::Update { name, r#ref } => run_update(name, r#ref, flags),
    }
}

pub(crate) struct AddArgs {
    pub url: String,
    pub source: Option<String>,
    pub plugin: Option<String>,
    pub skill: Option<String>,
    pub name: Option<String>,
    pub r#ref: Option<String>,
    pub symlink: bool,
    pub copy: bool,
}

pub(crate) fn run_add(args: AddArgs, flags: &Flags) -> anyhow::Result<()> {
    let AddArgs {
        url,
        source,
        plugin,
        skill,
        name,
        r#ref,
        symlink,
        copy,
    } = args;

    // Backward-compat: error on deprecated --name
    if name.is_some() {
        anyhow::bail!("`--name` has been renamed to `--source`");
    }

    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    let source_url = crate::source::SourceUrl::parse(&url)?;
    let effective_ref = r#ref.as_deref().or_else(|| source_url.tree_ref());
    let staged = stage_source_for_parse(&source_url, effective_ref, &data_dir)?;

    let residence = match &source_url {
        crate::source::SourceUrl::Git(..) | crate::source::SourceUrl::Archive(..) => {
            let kind_residence = crate::source::source_kind_residence(staged.parsed.kind);
            if kind_residence == crate::config::SourceResidence::External {
                // Marketplace → always external
                kind_residence
            } else if source.is_some() {
                // --source flag implies external
                crate::config::SourceResidence::External
            } else {
                crate::prompt::prompt_residence(flags.quiet)
            }
        }
        crate::source::SourceUrl::Local(_) => {
            let kind_residence = crate::source::source_kind_residence(staged.parsed.kind);
            if source.is_some() || kind_residence == crate::config::SourceResidence::External {
                crate::config::SourceResidence::External
            } else {
                crate::prompt::prompt_residence(flags.quiet)
            }
        }
    };

    if matches!(&source_url, crate::source::SourceUrl::Local(path) if path.is_file()) && symlink {
        anyhow::bail!("--symlink only works for local directory sources");
    }

    if matches!(source_url, crate::source::SourceUrl::Local(_))
        && residence == crate::config::SourceResidence::Local
        && (symlink || copy)
    {
        anyhow::bail!(
            "--symlink and --copy only apply to external local sources. Use --source <name> to register the path as an external source."
        );
    }

    let preview_source_name =
        preview_source_name(&source_url, &staged.parsed, source.as_deref(), residence);
    let preview = crate::source::normalize::normalize(
        &staged.parsed.clone().with_source_name(&preview_source_name),
    )?;

    if !flags.quiet {
        println!(
            "{}",
            detected_summary(
                &staged.parsed,
                &preview.plugins,
                &preview_source_name,
                &source_url.url_string(),
            )
        );
    }

    crate::prompt::confirm_proceed(flags.quiet)?;

    let source_name = match residence {
        crate::config::SourceResidence::External => {
            let default_source = default_source_name(&source_url, &staged.parsed);
            let source_name = if let Some(s) = source.clone() {
                s
            } else {
                crate::prompt::confirm_or_override("Source name", &default_source, flags.quiet)
            };

            if source_name == "local" {
                anyhow::bail!(
                    "'local' is reserved for the local plugin source. Use --source to choose a different name."
                );
            }

            if config
                .source
                .iter()
                .any(|existing| existing.id == source_name)
                || config.agent.iter().any(|a| a.id == source_name)
            {
                anyhow::bail!("id '{}' is already in use", source_name);
            }

            source_name
        }
        crate::config::SourceResidence::Local => "local".to_string(),
    };

    let overrides = prompt_add_overrides(
        &staged.parsed,
        source.clone(),
        plugin,
        skill,
        residence,
        flags,
    );
    let norm_overrides = crate::source::normalize::Overrides {
        plugin: overrides.0.as_deref(),
        skill: overrides.1.as_deref(),
    };

    let added = if flags.dry_run {
        None
    } else {
        Some(match residence {
            crate::config::SourceResidence::External => {
                let cache_path = crate::source::source_storage_path(&source_name, residence);
                let use_symlink = match &source_url {
                    crate::source::SourceUrl::Local(path) if path.is_dir() => {
                        if symlink {
                            true
                        } else if copy {
                            false
                        } else {
                            crate::prompt::prompt_fetch_mode(flags.quiet) == "symlink"
                        }
                    }
                    _ => false,
                };

                if !flags.quiet {
                    let action = match &source_url {
                        crate::source::SourceUrl::Git(url, _) => {
                            format!("Cloning {}", url.dimmed())
                        }
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

                let prepared = crate::source::prepare_source(
                    &source_name,
                    &source_url,
                    &cache_path,
                    r#ref.clone(),
                    if use_symlink {
                        Some("symlink".to_string())
                    } else {
                        None
                    },
                    residence,
                    &norm_overrides,
                )?;

                if !flags.quiet && !crate::prompt::is_interactive() {
                    for plugin in &prepared.registered.plugins {
                        for skill in &plugin.skills {
                            eprintln!(
                                "resolved: {}",
                                crate::output::plain_identity(
                                    &source_name,
                                    &plugin.name,
                                    &skill.name
                                )
                            );
                        }
                    }
                }

                let mut registry = crate::registry::load_registry(&data_dir)?;
                crate::source::persist_prepared_source(&mut config, &mut registry, prepared);
                crate::registry::save_registry(&registry, &data_dir)?;
                crate::config::save(&config, config_path_str)?;

                AddedSourceSummary::External { source_name }
            }
            crate::config::SourceResidence::Local => {
                let imported = crate::source::import_into_local_source(
                    &staged.parsed,
                    &norm_overrides,
                    &data_dir,
                )?;

                if !flags.quiet && !crate::prompt::is_interactive() {
                    for plugin in &imported.plugins {
                        for skill in &plugin.skills {
                            eprintln!(
                                "resolved: {}",
                                crate::output::plain_identity(
                                    &imported.source_name,
                                    &plugin.name,
                                    &skill.name
                                )
                            );
                        }
                    }
                }

                let registry =
                    crate::cli::helpers::load_effective_registry(&config, &data_dir, flags.quiet)?;
                crate::registry::save_registry(&registry, &data_dir)?;

                AddedSourceSummary::Local(imported)
            }
        })
    };

    if !flags.quiet {
        match added {
            Some(AddedSourceSummary::External { source_name }) => {
                let reg = crate::registry::load_registry(&data_dir)?;
                if let Some(src) = reg.sources.iter().find(|s| s.id == source_name) {
                    print_add_summary(
                        &format!("Added source {}", source_name.bold()),
                        &src.plugins,
                        r#ref.as_deref(),
                        flags.verbose,
                    );
                } else {
                    println!("{} Added source {}", "✓".green(), source_name.bold());
                }
            }
            Some(AddedSourceSummary::Local(imported)) => {
                let label = imported.display_name.as_deref().unwrap_or("local");
                let heading = if label == "local" {
                    "Imported into local source".to_string()
                } else {
                    format!(
                        "Imported into local source {}",
                        format!("({label})").dimmed()
                    )
                };
                print_add_summary(&heading, &imported.plugins, None, flags.verbose);
            }
            None => {}
        }
    }
    Ok(())
}

fn stage_source_for_parse(
    source_url: &crate::source::SourceUrl,
    git_ref: Option<&str>,
    data_dir: &std::path::Path,
) -> anyhow::Result<StagedParsedSource> {
    match source_url {
        crate::source::SourceUrl::Local(path) => Ok(StagedParsedSource {
            parsed: crate::source::ParsedSource::parse(&crate::source::detect_path(
                source_url, path,
            ))?,
            _temp_dir: None,
        }),
        crate::source::SourceUrl::Git(..) | crate::source::SourceUrl::Archive(..) => {
            let temp_dir = tempfile::Builder::new()
                .prefix("equip-add-")
                .tempdir_in(crate::config::internal_dir().parent().unwrap_or(data_dir))?;
            let staged_root = temp_dir.path().join("source");
            crate::source::fetch::fetch(source_url, &staged_root, git_ref)?;
            Ok(StagedParsedSource {
                parsed: crate::source::ParsedSource::parse(&crate::source::detect_path(
                    source_url,
                    &staged_root,
                ))?,
                _temp_dir: Some(temp_dir),
            })
        }
    }
}

fn prompt_add_overrides(
    parsed: &crate::source::ParsedSource,
    source: Option<String>,
    plugin: Option<String>,
    skill: Option<String>,
    residence: crate::config::SourceResidence,
    flags: &Flags,
) -> (Option<String>, Option<String>) {
    let source_as_plugin = if residence == crate::config::SourceResidence::Local {
        source
    } else {
        None
    };

    let plugin_override = if let Some(plugin) = plugin {
        Some(plugin)
    } else if let Some(source_plugin) = source_as_plugin {
        Some(source_plugin)
    } else if let Some(default_plugin) = parsed.prompt_plugin_name() {
        let confirmed =
            crate::prompt::confirm_or_override("Plugin name", default_plugin, flags.quiet);
        if confirmed != default_plugin {
            Some(confirmed)
        } else {
            None
        }
    } else {
        None
    };

    let skill_override = if let Some(skill) = skill {
        Some(skill)
    } else if let Some(default_skill) = parsed.prompt_skill_name() {
        let confirmed =
            crate::prompt::confirm_or_override("Skill name", default_skill, flags.quiet);
        if confirmed != default_skill {
            Some(confirmed)
        } else {
            None
        }
    } else {
        None
    };

    (plugin_override, skill_override)
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn print_add_summary(
    heading: &str,
    plugins: &[crate::registry::RegisteredPlugin],
    git_ref: Option<&str>,
    verbose: bool,
) {
    let plugin_count = plugins.len();
    let skill_count: usize = plugins.iter().map(|plugin| plugin.skills.len()).sum();

    println!(
        "{} {} {} {}",
        "✓".green(),
        heading,
        format!(
            "({} plugin{}, {} skill{})",
            plugin_count,
            plural(plugin_count),
            skill_count,
            plural(skill_count),
        )
        .dimmed(),
        git_ref
            .map(|value| format!("@ {}", value.cyan()))
            .unwrap_or_default(),
    );

    if !verbose {
        return;
    }

    for plugin in plugins {
        println!("  {} {}", "├──".dimmed(), plugin.name.green());
        for (index, skill) in plugin.skills.iter().enumerate() {
            let connector = if index == plugin.skills.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            let desc = skill.description.as_deref().unwrap_or("");
            if desc.is_empty() {
                println!("  {}   {} {}", "│".dimmed(), connector.dimmed(), skill.name);
            } else {
                println!(
                    "  {}   {} {} {}",
                    "│".dimmed(),
                    connector.dimmed(),
                    skill.name,
                    format!("— {}", desc).dimmed(),
                );
            }
        }
    }
}

fn preview_source_name(
    source_url: &crate::source::SourceUrl,
    parsed: &crate::source::ParsedSource,
    source: Option<&str>,
    residence: crate::config::SourceResidence,
) -> String {
    match residence {
        crate::config::SourceResidence::External => source
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| default_source_name(source_url, parsed)),
        crate::config::SourceResidence::Local => "local".to_string(),
    }
}

fn default_source_name(
    source_url: &crate::source::SourceUrl,
    parsed: &crate::source::ParsedSource,
) -> String {
    match parsed.kind {
        crate::source::SourceKind::Marketplace => {
            let url_alias = source_url.default_name();
            match parsed
                .display_name
                .as_deref()
                .map(source_alias)
                .filter(|name| !name.is_empty())
            {
                Some(display_alias) if display_alias == url_alias => display_alias,
                Some(display_alias) if display_alias.ends_with(&format!("-{url_alias}")) => {
                    display_alias
                }
                Some(display_alias) => format!("{display_alias}-{url_alias}"),
                None => url_alias,
            }
        }
        _ => source_url.default_name(),
    }
}

fn source_alias(name: &str) -> String {
    let mut alias = String::with_capacity(name.len());
    let mut last_was_hyphen = false;

    for ch in name.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else {
            Some('-')
        };

        if let Some(ch) = normalized {
            if ch == '-' {
                if alias.is_empty() || last_was_hyphen {
                    continue;
                }
                last_was_hyphen = true;
            } else {
                last_was_hyphen = false;
            }
            alias.push(ch);
        }
    }

    alias.trim_matches('-').to_string()
}

fn detected_summary(
    parsed: &crate::source::ParsedSource,
    plugins: &[crate::registry::RegisteredPlugin],
    source_name: &str,
    source_url: &str,
) -> String {
    let plugin_count = plugins.len();
    let skill_count: usize = plugins.iter().map(|plugin| plugin.skills.len()).sum();

    let mut lines = Vec::new();

    // Header line with kind and counts
    let header = match parsed.kind {
        crate::source::SourceKind::Marketplace => {
            let name = parsed
                .display_name
                .as_deref()
                .unwrap_or(&parsed.source_name);
            format!(
                "Detected marketplace {} — {} plugin{}, {} skill{}",
                name.cyan().bold(),
                plugin_count,
                plural(plugin_count),
                skill_count,
                plural(skill_count),
            )
        }
        crate::source::SourceKind::SinglePlugin => format!(
            "Detected {} plugin{} with {} skill{}",
            plugin_count,
            plural(plugin_count),
            skill_count,
            plural(skill_count),
        ),
        crate::source::SourceKind::FlatSkills
        | crate::source::SourceKind::SingleSkillDir
        | crate::source::SourceKind::SingleFile => format!(
            "Detected {} skill{}",
            skill_count,
            plural(skill_count),
        ),
    };
    lines.push(header);

    // Source and URL
    lines.push(format!(
        "  {} {}  {} {}",
        "source:".dimmed(),
        source_name.cyan(),
        "url:".dimmed(),
        source_url.dimmed(),
    ));

    // Skill listing
    lines.push(String::new());
    for plugin in plugins {
        let show_plugin = plugins.len() > 1 || plugin.name != source_name;
        if show_plugin {
            lines.push(format!("  {} {}", "plugin:".dimmed(), plugin.name.green()));
        }

        let indent = if show_plugin { "    " } else { "  " };

        for (i, skill) in plugin.skills.iter().enumerate() {
            let is_last = i == plugin.skills.len() - 1;
            let connector = if is_last { "└──" } else { "├──" };
            let desc = skill
                .description
                .as_deref()
                .filter(|d| !d.is_empty())
                .map(|d| format!(" {} {}", "—".dimmed(), d.dimmed()))
                .unwrap_or_default();
            lines.push(format!(
                "{}{} {}{}",
                indent,
                connector.dimmed(),
                skill.name.bold(),
                desc,
            ));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{default_source_name, detected_summary, source_alias, source_remove_target};
    use crate::registry::{RegisteredPlugin, RegisteredSkill};
    use crate::source::{ParsedSource, SourceKind, SourceUrl};
    use std::path::PathBuf;

    fn plugin(name: &str, skills: &[&str]) -> RegisteredPlugin {
        RegisteredPlugin {
            name: name.to_string(),
            version: None,
            description: None,
            path: PathBuf::from(name),
            skills: skills
                .iter()
                .map(|skill| RegisteredSkill {
                    name: (*skill).to_string(),
                    description: None,
                    author: None,
                    version: None,
                    path: PathBuf::from(format!("{name}/{skill}")),
                })
                .collect(),
        }
    }

    fn parsed(kind: SourceKind) -> ParsedSource {
        ParsedSource {
            kind,
            source_name: "src".to_string(),
            display_name: None,
            url: None,
            path: PathBuf::from("/tmp/src"),
            plugin_name: None,
            skill_name: None,
        }
    }

    #[test]
    fn marketplace_summary_includes_skills_and_source() {
        let mut parsed = parsed(SourceKind::Marketplace);
        parsed.display_name = Some("Team Skills".to_string());

        let summary = detected_summary(
            &parsed,
            &[plugin("alpha", &["one", "two"]), plugin("beta", &["three"])],
            "team-skills",
            "https://github.com/org/team-skills.git",
        );

        assert!(summary.contains("2 plugins, 3 skills"));
        assert!(summary.contains("team-skills"));
        assert!(summary.contains("https://github.com/org/team-skills.git"));
        assert!(summary.contains("one"));
        assert!(summary.contains("two"));
        assert!(summary.contains("three"));
        assert!(summary.contains("alpha"));
        assert!(summary.contains("beta"));
    }

    #[test]
    fn plugin_summary_lists_skills() {
        let summary = detected_summary(
            &parsed(SourceKind::SinglePlugin),
            &[plugin("alpha", &["one", "two"])],
            "alpha",
            "https://github.com/org/alpha.git",
        );

        assert!(summary.contains("1 plugin with 2 skills"));
        assert!(summary.contains("one"));
        assert!(summary.contains("two"));
        assert!(summary.contains("alpha"));
    }

    #[test]
    fn flat_skill_summary_lists_skills() {
        let summary = detected_summary(
            &parsed(SourceKind::FlatSkills),
            &[plugin("alpha", &["one", "two", "three"])],
            "alpha",
            "/tmp/alpha",
        );

        assert!(summary.contains("3 skills"));
        assert!(summary.contains("one"));
        assert!(summary.contains("two"));
        assert!(summary.contains("three"));
    }

    #[test]
    fn single_skill_summary_lists_skill() {
        let summary = detected_summary(
            &parsed(SourceKind::SingleSkillDir),
            &[plugin("alpha", &["one"])],
            "alpha",
            "/tmp/alpha",
        );

        assert!(summary.contains("1 skill"));
        assert!(summary.contains("one"));
    }

    #[test]
    fn marketplace_default_source_name_uses_marketplace_name() {
        let mut parsed = parsed(SourceKind::Marketplace);
        parsed.display_name = Some("HashiCorp Skills".to_string());
        let url = SourceUrl::parse("https://github.com/hashicorp/agent-skills").unwrap();

        assert_eq!(
            default_source_name(&url, &parsed),
            "hashicorp-skills-agent-skills"
        );
    }

    #[test]
    fn marketplace_default_source_name_falls_back_to_url_when_name_missing() {
        let parsed = parsed(SourceKind::Marketplace);
        let url = SourceUrl::parse("https://github.com/hashicorp/agent-skills").unwrap();

        assert_eq!(default_source_name(&url, &parsed), "agent-skills");
    }

    #[test]
    fn source_alias_normalizes_human_names() {
        assert_eq!(
            source_alias("HashiCorp Agent Skills"),
            "hashicorp-agent-skills"
        );
        assert_eq!(source_alias(" team__skills "), "team-skills");
    }

    #[test]
    fn source_remove_target_prefers_exact_source_name() {
        assert!(source_remove_target("team-skills", true));
        assert!(!source_remove_target("team-skills/*", true));
        assert!(!source_remove_target("local:team-skills/*", true));
        assert!(!source_remove_target("team-*", true));
        assert!(!source_remove_target("team-skills", false));
    }
}

pub(crate) fn run_list(
    patterns: Vec<String>,
    external: bool,
    fzf: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    if external {
        return run_source_list(flags);
    }

    let ctx = load_context(flags)?;

    // Collect matching skills from patterns
    let skills: Vec<(
        &str,
        &crate::registry::RegisteredPlugin,
        &crate::registry::RegisteredSkill,
    )> = if patterns.is_empty() {
        ctx.registry.all_skills()
    } else {
        resolve_skill_patterns(&patterns, &ctx.registry, true)?
    };

    // Interactive fzf mode
    if fzf {
        if skills.is_empty() {
            let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
            output.info("No skills to browse.");
            return Ok(());
        }

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
                    anyhow::anyhow!(
                        "fzf not found in PATH. Install fzf: https://github.com/junegunn/fzf"
                    )
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

        let status = skill_list_status(source_name, plugin, skill, &ctx);

        if flags.json {
            let json = serde_json::json!({
                "identity": crate::output::plain_identity(source_name, plugin_name, &skill.name),
                "name": skill.name,
                "plugin": plugin_name,
                "source": source_name,
                "description": skill.description,
                "path": skill.path,
                "status": status.label(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
            return Ok(());
        }

        let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        out.status(
            "Identity",
            &crate::output::format_identity(source_name, plugin_name, &skill.name),
        );
        out.status(
            "Description",
            skill.description.as_deref().unwrap_or("(none)"),
        );
        if status != SkillListStatus::Clean {
            let status_display = match status {
                SkillListStatus::Untracked => format!("{}", "untracked".yellow()),
                SkillListStatus::Modified => format!("{}", "modified".red()),
                SkillListStatus::Clean => "clean".to_string(),
            };
            out.status("Status", &status_display);
        }
        if flags.verbose {
            out.status("Path", &skill.path.display().to_string());
        }
    } else if flags.json {
        let entries: Vec<serde_json::Value> = skills.iter()
            .map(|(source_name, plugin, skill)| {
                let source_ref = ctx.config.source.iter()
                    .find(|cs| cs.id == *source_name)
                    .and_then(|cs| cs.r#ref.clone());
                let status = skill_list_status(source_name, plugin, skill, &ctx);
                let mut entry = serde_json::json!({
                    "identity": crate::output::plain_identity(source_name, &plugin.name, &skill.name),
                    "name": skill.name,
                    "plugin": plugin.name,
                    "source": source_name,
                    "status": status.label(),
                });
                if let Some(ref r) = source_ref {
                    entry["ref"] = serde_json::Value::String(r.clone());
                }
                entry
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        if skills.is_empty() {
            if patterns.is_empty() {
                output.info("No skills found. Add a source with `equip add`");
            } else {
                output.info("No skills matched the given pattern(s)");
            }
        } else {
            for (source_name, plugin, skill) in &skills {
                let status = skill_list_status(source_name, plugin, skill, &ctx);
                let identity =
                    crate::output::format_identity(source_name, &plugin.name, &skill.name);
                let prefix = match status {
                    SkillListStatus::Untracked => format!("{}", "?".yellow()),
                    SkillListStatus::Modified => format!("{}", "m".red()),
                    SkillListStatus::Clean => " ".to_string(),
                };
                println!("{} {}", prefix, identity);
            }
        }
    }
    Ok(())
}

pub(crate) fn run_source_list(flags: &Flags) -> anyhow::Result<()> {
    let ctx = load_context(flags)?;
    let config = ctx.config;
    let registry = ctx.registry;

    if flags.json {
        let entries: Vec<serde_json::Value> = config
            .source
            .iter()
            .map(|src| {
                let skill_count: usize = registry
                    .sources
                    .iter()
                    .find(|rs| rs.id == src.id)
                    .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                    .unwrap_or(0);
                serde_json::json!({
                    "name": src.id,
                    "type": src.source_type,
                    "residence": src.residence.as_str(),
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

    if config.source.is_empty() {
        let output = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
        output.info("No sources configured. Use `equip add` to add one.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = config
        .source
        .iter()
        .map(|src| {
            let skill_count: usize = registry
                .sources
                .iter()
                .find(|rs| rs.id == src.id)
                .map(|rs| rs.plugins.iter().map(|p| p.skills.len()).sum())
                .unwrap_or(0);
            vec![
                src.id.clone(),
                src.source_type.clone(),
                src.residence.as_str().to_string(),
                extract_domain(&src.url),
                src.r#ref.clone().unwrap_or_default(),
                skill_count.to_string(),
                src.mode.clone().unwrap_or_default(),
            ]
        })
        .collect();

    let out = crate::output::Output::from_flags(flags.json, flags.quiet, flags.verbose);
    out.table(
        &["NAME", "TYPE", "STORAGE", "DOMAIN", "REF", "SKILLS", "MODE"],
        &rows,
    );
    Ok(())
}

enum RemovalAction {
    Execute,
    Preview,
    Abort,
}

fn removal_action(label: &str, force: bool, flags: &Flags) -> RemovalAction {
    if flags.dry_run {
        return RemovalAction::Preview;
    }
    if force {
        return RemovalAction::Execute;
    }
    if crate::prompt::confirm_action(label, flags.quiet, false) {
        RemovalAction::Execute
    } else if flags.quiet || !crate::prompt::is_interactive() {
        RemovalAction::Preview
    } else {
        RemovalAction::Abort
    }
}

fn source_remove_target(pattern: &str, source_exists: bool) -> bool {
    source_exists
        && !pattern.contains('/')
        && !pattern.contains(':')
        && !crate::registry::is_glob(pattern)
}

pub(crate) fn run_remove(patterns: Vec<String>, force: bool, flags: &Flags) -> anyhow::Result<()> {
    if patterns.is_empty() {
        anyhow::bail!("remove requires a local skill pattern or `equip source remove <name>`");
    }

    let config = crate::config::load(flags.config_path())?;
    let data_dir = crate::config::data_dir();
    let registry = crate::cli::helpers::load_effective_registry(&config, &data_dir, flags.quiet)?;

    let source_match = if patterns.len() == 1 {
        config.source.iter().any(|source| source.id == patterns[0])
    } else {
        false
    };

    if patterns.len() == 1 && source_remove_target(&patterns[0], source_match) {
        return run_source_remove(Some(patterns[0].clone()), force, flags);
    }

    let resolved = crate::cli::helpers::resolve_skill_patterns(&patterns, &registry, true);
    match resolved {
        Ok(skills) => {
            if patterns.len() == 1 && source_match {
                if flags.quiet || !crate::prompt::is_interactive() {
                    anyhow::bail!(
                        "remove target '{}' is ambiguous. Use a fully qualified skill identity or `equip source remove {}`.",
                        patterns[0],
                        patterns[0]
                    );
                }

                let choices = vec![
                    "Remove matching local skill(s)".to_string(),
                    format!("Remove source '{}'", patterns[0]),
                ];
                let selected = crate::prompt::select_from("Remove target", &choices, flags.quiet)?;
                if selected == choices[1] {
                    return run_source_remove(Some(patterns[0].clone()), force, flags);
                }
            }

            remove_local_skills(skills, force, &registry, &data_dir, flags)
        }
        Err(skill_error) => {
            if source_match {
                run_source_remove(Some(patterns[0].clone()), force, flags)
            } else {
                Err(skill_error)
            }
        }
    }
}

fn remove_local_skills(
    skills: Vec<(
        &str,
        &crate::registry::RegisteredPlugin,
        &crate::registry::RegisteredSkill,
    )>,
    force: bool,
    registry: &crate::registry::Registry,
    data_dir: &std::path::Path,
    flags: &Flags,
) -> anyhow::Result<()> {
    if skills.is_empty() {
        anyhow::bail!("no local skills matched the given pattern(s)");
    }

    let external_matches: Vec<String> = skills
        .iter()
        .filter(|(source_name, _, _)| *source_name != "local")
        .map(|(source_name, plugin, skill)| {
            crate::output::plain_identity(source_name, &plugin.name, &skill.name)
        })
        .collect();
    if !external_matches.is_empty() {
        anyhow::bail!(
            "remove only supports skills from the local source. Use `equip source remove <name>` for external sources.\n{}",
            external_matches.join("\n")
        );
    }

    let mut installed_on = std::collections::BTreeSet::new();
    for (agent_name, installed_skills) in &registry.installed {
        let matches_installed = skills.iter().any(|(_, plugin, skill)| {
            installed_skills.values().any(|installed| {
                installed.source == "local"
                    && installed.plugin == plugin.name
                    && installed.skill == skill.name
            })
        });
        if matches_installed {
            installed_on.insert(agent_name.clone());
        }
    }

    if !installed_on.is_empty() && !flags.quiet {
        eprintln!(
            "warning: local skill(s) are installed on: {}",
            installed_on.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    let action = removal_action(
        &format!(
            "Remove {} local skill{}?",
            skills.len(),
            if skills.len() == 1 { "" } else { "s" }
        ),
        force,
        flags,
    );

    match action {
        RemovalAction::Abort => {
            if !flags.quiet {
                eprintln!("Aborted.");
            }
            return Ok(());
        }
        RemovalAction::Preview => {
            if !flags.quiet {
                println!(
                    "Would remove {} local skill{}",
                    skills.len(),
                    if skills.len() == 1 { "" } else { "s" }
                );
                println!("Use --force to remove, or run interactively to confirm.");
            }
            return Ok(());
        }
        RemovalAction::Execute => {}
    }

    let mut affected_plugins = std::collections::BTreeSet::new();
    for (_, plugin, skill) in &skills {
        affected_plugins.insert(plugin.path.clone());
        if skill.path.exists() {
            std::fs::remove_dir_all(&skill.path)?;
        }
    }

    for plugin_path in affected_plugins {
        let skills_dir = plugin_path.join("skills");
        let has_skills = std::fs::read_dir(&skills_dir)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && entry.path().join("SKILL.md").exists()
                })
            })
            .unwrap_or(false);
        if !has_skills && plugin_path.exists() {
            std::fs::remove_dir_all(&plugin_path)?;
        }
    }

    crate::marketplace::generate_local_manifest(data_dir)?;

    let config = crate::config::load(flags.config_path())?;
    let updated_registry =
        crate::cli::helpers::load_effective_registry(&config, data_dir, flags.quiet)?;
    crate::registry::save_registry(&updated_registry, data_dir)?;

    if !flags.quiet {
        println!(
            "Removed {} local skill{}",
            skills.len(),
            if skills.len() == 1 { "" } else { "s" }
        );
    }
    Ok(())
}

pub(crate) fn run_source_remove(
    name: Option<String>,
    force: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();

    let name = match name {
        Some(n) => n,
        None => {
            let source_names: Vec<String> = config.source.iter().map(|s| s.id.clone()).collect();
            if source_names.is_empty() {
                anyhow::bail!("no sources configured");
            }
            crate::prompt::select_from("Select source to remove", &source_names, flags.quiet)?
        }
    };

    if !config.source.iter().any(|s| s.id == name) {
        anyhow::bail!("source '{}' not found", name);
    }

    // Check if any skills from this source are installed on agents
    let registry = crate::registry::load_registry(&data_dir)?;
    let mut installed_on: Vec<String> = Vec::new();
    if let Some(reg_src) = registry.sources.iter().find(|s| s.id == name) {
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
                            installed_on.push(ac.id.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    if !installed_on.is_empty() && !flags.quiet {
        eprintln!(
            "warning: source '{}' has installed skills on: {}",
            name,
            installed_on.join(", ")
        );
    }

    match removal_action(&format!("Remove source '{}'?", name), force, flags) {
        RemovalAction::Abort => {
            if !flags.quiet {
                eprintln!("Aborted.");
            }
            return Ok(());
        }
        RemovalAction::Preview => {
            if !flags.quiet {
                println!("Would remove source '{}'", name);
                println!("Use --force to remove, or run interactively to confirm.");
            }
            return Ok(());
        }
        RemovalAction::Execute => {}
    }

    {
        // Remove cached content
        let residence = config
            .source
            .iter()
            .find(|s| s.id == name)
            .map(|s| s.residence)
            .unwrap_or(crate::source::default_source_residence());
        let cache_path = crate::source::source_storage_path(&name, residence);
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)?;
        }

        // Remove from registry
        let mut registry = crate::registry::load_registry(&data_dir)?;
        registry.sources.retain(|s| s.id != name);
        crate::registry::save_registry(&registry, &data_dir)?;

        // Remove from config
        config.source.retain(|s| s.id != name);
        // Clean equipped entries referencing the removed source
        let prefix = format!("{}:", name);
        for ac in &mut config.agent {
            ac.equipped.retain(|e| !e.starts_with(&prefix));
        }
        crate::config::save(&config, config_path_str)?;
    }

    if !flags.quiet {
        println!("Removed source '{}'", name);
    }
    Ok(())
}

pub(crate) fn run_update(
    name: Option<String>,
    update_ref: Option<String>,
    flags: &Flags,
) -> anyhow::Result<()> {
    let config_path_str = flags.config_path();
    let mut config = crate::config::load(config_path_str)?;
    let data_dir = crate::config::data_dir();
    let mut registry = crate::registry::load_registry(&data_dir)?;
    let renames = crate::registry::reconcile_with_config(&mut registry, &config.source, &data_dir)?;
    if !renames.is_empty() {
        crate::registry::save_registry(&registry, &data_dir)?;
        if !flags.quiet {
            for r in &renames {
                eprintln!("source reconciled: {}", r);
            }
        }
    }

    if update_ref.is_some() && name.is_none() {
        anyhow::bail!("--ref requires a source name (e.g., equip update my-source --ref v2.0)");
    }

    // Determine which sources to update
    let sources_to_update: Vec<crate::config::SourceConfig> = if let Some(ref n) = name {
        let src = config
            .source
            .iter()
            .find(|s| s.id == *n)
            .ok_or_else(|| anyhow::anyhow!("source '{}' not found", n))?;
        vec![src.clone()]
    } else {
        if config.source.is_empty() {
            if !flags.quiet {
                println!("No sources to update.");
            }
            return Ok(());
        }
        config.source.clone()
    };

    let mut updated_registry = registry;
    let mut updated_count = 0;
    let mut errors = Vec::new();
    let mut ref_changed = false;

    for src in &sources_to_update {
        if !flags.quiet {
            println!("Updating '{}'...", src.id);
        }

        if flags.dry_run {
            if !flags.quiet {
                println!("  (dry run) would re-fetch from {}", src.url);
            }
            updated_count += 1;
            continue;
        }

        let cache_path = crate::source::source_storage_path_for_config(src);
        if src.mode.as_deref() == Some("symlink") && !flags.quiet {
            println!("  (symlinked, re-detecting)");
        }

        match crate::source::refresh_source(src, &cache_path, update_ref.as_deref()) {
            Ok(crate::source::RefreshSource::Updated(prepared)) => {
                if update_ref.is_some()
                    && prepared.config.r#ref != src.r#ref
                    && prepared.config.source_type == "git"
                {
                    ref_changed = true;
                }
                crate::source::persist_prepared_source(
                    &mut config,
                    &mut updated_registry,
                    *prepared,
                );
                updated_count += 1;
            }
            Ok(crate::source::RefreshSource::SkippedPinned { pinned_ref }) => {
                if !flags.quiet {
                    eprintln!(
                        "warning: source '{}' is pinned to {}, skipping",
                        src.id, pinned_ref
                    );
                }
                continue;
            }
            Err(e) => {
                errors.push(format!("{}: {}", src.id, e));
                continue;
            }
        }
    }

    if !flags.dry_run {
        crate::registry::save_registry(&updated_registry, &data_dir)?;
        if ref_changed {
            if let Some(ref new_ref) = update_ref {
                if let Some(ref source_name) = name {
                    if let Some(cfg_src) = config.source.iter_mut().find(|s| s.id == *source_name) {
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

    if !flags.quiet {
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

pub(crate) fn run_reconcile(
    source_name: Option<String>,
    rewrite_config: bool,
    flags: &Flags,
) -> anyhow::Result<()> {
    if rewrite_config {
        let config_path = crate::config::config_path(flags.config_path());
        if config_path.exists() {
            let config = crate::config::load(flags.config_path())?;
            if flags.dry_run {
                if !flags.quiet {
                    println!("Would rewrite {}", config_path.display());
                }
            } else {
                crate::config::save_to(&config, &config_path)?;
                if !flags.quiet {
                    println!("Rewrote {}", config_path.display());
                }
            }
        } else if !flags.quiet {
            println!("No config file to rewrite");
        }
    }

    let config = crate::config::load(flags.config_path())?;
    let data_dir = crate::config::data_dir();
    let mut registry = crate::registry::load_registry(&data_dir)?;
    let renames = crate::registry::reconcile_with_config(&mut registry, &config.source, &data_dir)?;
    if !renames.is_empty() {
        crate::registry::save_registry(&registry, &data_dir)?;
        if !flags.quiet {
            for rename in &renames {
                eprintln!("source reconciled: {}", rename);
            }
        }
    }

    let mut refreshed_sources = 0usize;
    let mut updated_origins = 0usize;
    let mut warnings = Vec::new();

    let reconcile_local = source_name
        .as_deref()
        .map(|name| name == "local")
        .unwrap_or(true);

    if source_name.as_deref() != Some("local") {
        let sources_to_reconcile: Vec<_> = if let Some(ref name) = source_name {
            let source = config
                .source
                .iter()
                .find(|source| source.id == *name)
                .ok_or_else(|| anyhow::anyhow!("source '{}' not found", name))?;
            vec![source.clone()]
        } else {
            config.source.clone()
        };

        for source in &sources_to_reconcile {
            let old_source = registry
                .sources
                .iter()
                .find(|registered| registered.id == source.id)
                .cloned();
            let cache_path = crate::source::source_storage_path_for_config(source);
            if !cache_path.exists() {
                warnings.push(format!(
                    "source '{}' cache path is missing: {}",
                    source.id,
                    cache_path.display()
                ));
                continue;
            }

            match reconcile_registered_source(source, &cache_path) {
                Ok(updated_source) => {
                    updated_origins += reconcile_installed_origins(
                        &mut registry,
                        &data_dir,
                        old_source.as_ref(),
                        &updated_source,
                    );
                    registry
                        .sources
                        .retain(|registered| registered.id != updated_source.id);
                    registry.sources.push(updated_source);
                    refreshed_sources += 1;
                }
                Err(err) => warnings.push(format!("{}: {}", source.id, err)),
            }
        }
    }

    if reconcile_local {
        if !flags.dry_run {
            crate::cli::helpers::generate_marketplace(&data_dir)?;
        }

        let old_local = registry
            .sources
            .iter()
            .find(|registered| registered.id == "local")
            .cloned();

        match reconcile_local_source(&data_dir) {
            Ok(Some(local_source)) => {
                updated_origins += reconcile_installed_origins(
                    &mut registry,
                    &data_dir,
                    old_local.as_ref(),
                    &local_source,
                );
                registry
                    .sources
                    .retain(|registered| registered.id != local_source.id);
                registry.sources.push(local_source);
                refreshed_sources += 1;
            }
            Ok(None) => {
                registry
                    .sources
                    .retain(|registered| registered.id != "local");
            }
            Err(err) => warnings.push(format!("local: {}", err)),
        }
    }

    if !flags.dry_run {
        crate::registry::save_registry(&registry, &data_dir)?;
    }

    if !flags.quiet {
        if flags.dry_run {
            println!(
                "Would reconcile {} source(s); {} installed origin record(s) would change",
                refreshed_sources, updated_origins
            );
        } else {
            println!(
                "Reconciled {} source(s); updated {} installed origin record(s)",
                refreshed_sources, updated_origins
            );
        }
        for warning in &warnings {
            eprintln!("warning: {}", warning);
        }
    }

    Ok(())
}

fn reconcile_registered_source(
    source: &crate::config::SourceConfig,
    cache_path: &std::path::Path,
) -> anyhow::Result<crate::registry::RegisteredSource> {
    let source_url = crate::source::SourceUrl::parse(&source.url)?;
    let prepared = crate::source::prepare_source(
        &source.id,
        &source_url,
        cache_path,
        source.r#ref.clone(),
        source.mode.clone(),
        source.residence,
        &crate::source::normalize::Overrides::default(),
    )?;
    Ok(prepared.registered)
}

fn reconcile_local_source(
    data_dir: &std::path::Path,
) -> anyhow::Result<Option<crate::registry::RegisteredSource>> {
    let parsed = match crate::source::ParsedSource::parse(data_dir) {
        Ok(parsed) => parsed,
        Err(_) => return Ok(None),
    };
    if parsed.kind != crate::source::SourceKind::Marketplace {
        return Ok(None);
    }

    let mut local =
        crate::source::normalize::normalize(&parsed.with_source_name("local").with_url(""))?;
    local.id = "local".to_string();
    local.url.clear();
    local.residence = crate::config::SourceResidence::Local;
    Ok(Some(local))
}

fn reconcile_installed_origins(
    registry: &mut crate::registry::Registry,
    data_dir: &std::path::Path,
    old_source: Option<&crate::registry::RegisteredSource>,
    new_source: &crate::registry::RegisteredSource,
) -> usize {
    let Some(old_source) = old_source else {
        return 0;
    };

    let mut old_paths = std::collections::BTreeMap::new();
    for plugin in &old_source.plugins {
        for skill in &plugin.skills {
            old_paths.insert(
                (plugin.name.clone(), skill.name.clone()),
                relative_storage_path(data_dir, &skill.path),
            );
        }
    }

    let mut new_paths = std::collections::BTreeMap::new();
    for plugin in &new_source.plugins {
        for skill in &plugin.skills {
            new_paths.insert(
                (plugin.name.clone(), skill.name.clone()),
                relative_storage_path(data_dir, &skill.path),
            );
        }
    }

    let mut updated = 0usize;
    for installed in registry.installed.values_mut() {
        for entry in installed.values_mut() {
            if entry.source != new_source.id {
                continue;
            }
            let key = (entry.plugin.clone(), entry.skill.clone());
            let Some(old_origin) = old_paths.get(&key) else {
                continue;
            };
            let Some(new_origin) = new_paths.get(&key) else {
                continue;
            };
            if old_origin != new_origin && entry.origin == *old_origin {
                entry.origin = new_origin.clone();
                updated += 1;
            }
        }
    }

    updated
}

fn relative_storage_path(data_dir: &std::path::Path, path: &std::path::Path) -> String {
    match path.strip_prefix(data_dir) {
        Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
        Err(_) => path.to_string_lossy().replace('\\', "/"),
    }
}
