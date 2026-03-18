use clap::Parser;
use equip::cli::args::preprocess;
use equip::cli::Cli;

/// Verify clap parsing of global flags at the Rust level.
/// Full functional coverage is in Docker suite 00.

#[test]
fn parse_help_flag() {
    // --help causes clap to exit, so we check that parsing without it works
    let cli = Cli::try_parse_from(["equip", "status"]).unwrap();
    assert!(!cli.json);
    assert!(!cli.quiet);
    assert!(!cli.verbose);
    assert!(!cli.dry_run);
}

#[test]
fn parse_dry_run_flag() {
    let cli = Cli::try_parse_from(["equip", "-n", "status"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_dry_run_long_flag() {
    let cli = Cli::try_parse_from(["equip", "--dry-run", "status"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_json_flag() {
    let cli = Cli::try_parse_from(["equip", "--json", "status"]).unwrap();
    assert!(cli.json);
}

#[test]
fn parse_quiet_flag() {
    let cli = Cli::try_parse_from(["equip", "-q", "status"]).unwrap();
    assert!(cli.quiet);
}

#[test]
fn parse_verbose_flag() {
    let cli = Cli::try_parse_from(["equip", "-v", "status"]).unwrap();
    assert!(cli.verbose);
}

#[test]
fn parse_config_override() {
    let cli = Cli::try_parse_from(["equip", "--config", "/tmp/alt.toml", "status"]).unwrap();
    assert_eq!(cli.config, Some("/tmp/alt.toml".to_string()));
}

#[test]
fn parse_equip_parses_ok() {
    // equip with no flags should parse OK at clap level (error is in run())
    let result = Cli::try_parse_from(["equip", "_equip"]);
    assert!(result.is_ok());
}

#[test]
fn parse_agent_add() {
    let cli = Cli::try_parse_from([
        "equip", "agent", "add", "claude", "/tmp/t", "--scope", "repo", "--name", "my-agent",
    ])
    .unwrap();
    match cli.command {
        equip::cli::Command::Agent { command } => {
            match command {
                equip::cli::AgentCommand::Add {
                    agent,
                    path,
                    scope,
                    sync,
                    name,
                } => {
                    assert_eq!(agent, "claude");
                    assert_eq!(scope, "repo");
                    assert_eq!(name, Some("my-agent".to_string()));
                    assert!(path.is_some());
                    let _ = sync; // just verify it parsed
                }
                _ => panic!("expected Add"),
            }
        }
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_add() {
    let cli = Cli::try_parse_from(["equip", "add", "/tmp/src", "--source", "my-src"]).unwrap();
    match cli.command {
        equip::cli::Command::Add { url, source, .. } => {
            assert_eq!(url, "/tmp/src");
            assert_eq!(source, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_deprecated_name_flag() {
    // --name still parses (hidden flag) but the handler will bail with deprecation error
    let cli = Cli::try_parse_from(["equip", "add", "/tmp/src", "--name", "my-src"]).unwrap();
    match cli.command {
        equip::cli::Command::Add { name, .. } => {
            assert_eq!(name, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_with_plugin_and_skill_flags() {
    let cli = Cli::try_parse_from([
        "equip", "add", "/tmp/src", "--source", "s", "--plugin", "p", "--skill", "sk",
    ])
    .unwrap();
    match cli.command {
        equip::cli::Command::Add {
            source,
            plugin,
            skill,
            ..
        } => {
            assert_eq!(source, Some("s".to_string()));
            assert_eq!(plugin, Some("p".to_string()));
            assert_eq!(skill, Some("sk".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_symlink_flag() {
    let cli = Cli::try_parse_from(["equip", "add", "/tmp/src", "--symlink"]).unwrap();
    match cli.command {
        equip::cli::Command::Add { symlink, copy, .. } => {
            assert!(symlink);
            assert!(!copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_copy_flag() {
    let cli = Cli::try_parse_from(["equip", "add", "/tmp/src", "--copy"]).unwrap();
    match cli.command {
        equip::cli::Command::Add { symlink, copy, .. } => {
            assert!(!symlink);
            assert!(copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_symlink_copy_conflict() {
    let result = Cli::try_parse_from(["equip", "add", "/tmp/src", "--symlink", "--copy"]);
    assert!(result.is_err(), "--symlink and --copy should conflict");
}

#[test]
fn parse_remove_without_name() {
    let cli = Cli::try_parse_from(["equip", "remove"]).unwrap();
    match cli.command {
        equip::cli::Command::Remove { patterns, force } => {
            assert!(patterns.is_empty());
            assert!(!force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_list() {
    let cli = Cli::try_parse_from(["equip", "list"]).unwrap();
    match cli.command {
        equip::cli::Command::List { patterns, .. } => {
            assert!(patterns.is_empty());
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_name() {
    let cli = Cli::try_parse_from(["equip", "list", "test-plugin/explore"]).unwrap();
    match cli.command {
        equip::cli::Command::List { patterns, .. } => {
            assert_eq!(patterns, vec!["test-plugin/explore".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_multiple_patterns() {
    let cli = Cli::try_parse_from(["equip", "list", "legal/*", "sales/*"]).unwrap();
    match cli.command {
        equip::cli::Command::List { patterns, .. } => {
            assert_eq!(patterns, vec!["legal/*".to_string(), "sales/*".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_remove() {
    let cli = Cli::try_parse_from(["equip", "remove", "my-plugin/my-skill", "--force"]).unwrap();
    match cli.command {
        equip::cli::Command::Remove { patterns, force } => {
            assert_eq!(patterns, vec!["my-plugin/my-skill".to_string()]);
            assert!(force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_remove_multiple_patterns() {
    let cli = Cli::try_parse_from(["equip", "remove", "one/*", "two/*"]).unwrap();
    match cli.command {
        equip::cli::Command::Remove { patterns, force } => {
            assert_eq!(patterns, vec!["one/*".to_string(), "two/*".to_string()]);
            assert!(!force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_update() {
    let cli = Cli::try_parse_from(["equip", "source", "update", "my-source"]).unwrap();
    match cli.command {
        equip::cli::Command::Source { command } => match command {
            equip::cli::SourceCommand::Update { name, .. } => {
                assert_eq!(name, Some("my-source".to_string()));
            }
            _ => panic!("expected Source::Update"),
        },
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_update_all() {
    let cli = Cli::try_parse_from(["equip", "source", "update"]).unwrap();
    match cli.command {
        equip::cli::Command::Source { command } => match command {
            equip::cli::SourceCommand::Update { name, .. } => {
                assert!(name.is_none());
            }
            _ => panic!("expected Source::Update"),
        },
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_source_list() {
    let cli = Cli::try_parse_from(["equip", "source", "list"]).unwrap();
    match cli.command {
        equip::cli::Command::Source { command } => match command {
            equip::cli::SourceCommand::List => {}
            _ => panic!("expected Source::List"),
        },
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_source_add() {
    let cli =
        Cli::try_parse_from(["equip", "source", "add", "/tmp/src", "--source", "my-src"]).unwrap();
    match cli.command {
        equip::cli::Command::Source { command } => match command {
            equip::cli::SourceCommand::Add { url, source, .. } => {
                assert_eq!(url, "/tmp/src");
                assert_eq!(source, Some("my-src".to_string()));
            }
            _ => panic!("expected Source::Add"),
        },
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_source_remove() {
    let cli = Cli::try_parse_from(["equip", "source", "remove", "my-source", "--force"]).unwrap();
    match cli.command {
        equip::cli::Command::Source { command } => match command {
            equip::cli::SourceCommand::Remove { name, force } => {
                assert_eq!(name, Some("my-source".to_string()));
                assert!(force);
            }
            _ => panic!("expected Source::Remove"),
        },
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_top_level_update_rejected() {
    let cli = Cli::try_parse_from(["equip", "update"]);
    assert!(cli.is_err());
}

#[test]
fn parse_init_with_url() {
    let cli = Cli::try_parse_from(["equip", "init", "https://github.com/org/skills"]).unwrap();
    match cli.command {
        equip::cli::Command::Init { url } => {
            assert_eq!(url, Some("https://github.com/org/skills".to_string()));
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn parse_init_without_url() {
    let cli = Cli::try_parse_from(["equip", "init"]).unwrap();
    match cli.command {
        equip::cli::Command::Init { url } => {
            assert!(url.is_none());
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn parse_kit_create() {
    let cli = Cli::try_parse_from([
        "equip",
        "kit",
        "create",
        "dev",
        "plugin/skill-a",
        "plugin/skill-b",
    ])
    .unwrap();
    match cli.command {
        equip::cli::Command::Kit { command } => match command {
            equip::cli::KitCommand::Create { name, skills } => {
                assert_eq!(name, "dev");
                assert_eq!(
                    skills,
                    vec!["plugin/skill-a".to_string(), "plugin/skill-b".to_string()]
                );
            }
            _ => panic!("expected Create"),
        },
        _ => panic!("expected Kit"),
    }
}

#[test]
fn parse_kit_delete() {
    let cli = Cli::try_parse_from(["equip", "kit", "delete", "dev", "--force"]).unwrap();
    match cli.command {
        equip::cli::Command::Kit { command } => match command {
            equip::cli::KitCommand::Delete { name, force } => {
                assert_eq!(name, "dev");
                assert!(force);
            }
            _ => panic!("expected Delete"),
        },
        _ => panic!("expected Kit"),
    }
}

#[test]
fn parse_kit_show() {
    let cli = Cli::try_parse_from(["equip", "kit", "show", "dev"]).unwrap();
    match cli.command {
        equip::cli::Command::Kit { command } => match command {
            equip::cli::KitCommand::Show { name } => {
                assert_eq!(name, "dev");
            }
            _ => panic!("expected Show"),
        },
        _ => panic!("expected Kit"),
    }
}

#[test]
fn parse_kit_add_skills() {
    let cli = Cli::try_parse_from(["equip", "kit", "add", "dev", "plugin/skill-a"]).unwrap();
    match cli.command {
        equip::cli::Command::Kit { command } => match command {
            equip::cli::KitCommand::Add { name, skills } => {
                assert_eq!(name, "dev");
                assert_eq!(skills, vec!["plugin/skill-a".to_string()]);
            }
            _ => panic!("expected Add"),
        },
        _ => panic!("expected Kit"),
    }
}

#[test]
fn parse_kit_drop_skills() {
    let cli = Cli::try_parse_from(["equip", "kit", "drop", "dev", "plugin/skill-a"]).unwrap();
    match cli.command {
        equip::cli::Command::Kit { command } => match command {
            equip::cli::KitCommand::Drop { name, skills } => {
                assert_eq!(name, "dev");
                assert_eq!(skills, vec!["plugin/skill-a".to_string()]);
            }
            _ => panic!("expected Drop"),
        },
        _ => panic!("expected Kit"),
    }
}

#[test]
fn multi_select_returns_empty_non_interactive() {
    let result = equip::prompt::multi_select("Pick", &["a", "b"], &[true, true], false);
    assert!(result.is_empty());
}

#[test]
fn detect_agents_returns_vec() {
    // In a tempdir with no agent dirs, should return empty
    let result = equip::cli::detect_agents();
    // Can't assert empty because the test runner's home may have agents
    // Just verify it returns without error
    let _ = result;
}

// --- Shorthand argument syntax integration tests ---

fn pp(args: &[&str]) -> Vec<String> {
    preprocess(args.iter().map(|s| s.to_string()).collect())
}

#[test]
fn shorthand_top_level_at_parses() {
    let processed = pp(&["equip", "@claude", "dev*"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        equip::cli::Command::Equip {
            agent, patterns, ..
        } => {
            assert_eq!(agent, Some(vec!["claude".to_string()]));
            assert_eq!(patterns, vec!["dev*".to_string()]);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn shorthand_top_level_plus_parses() {
    let processed = pp(&["equip", "+developer"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        equip::cli::Command::Equip { kit, .. } => {
            assert_eq!(kit, Some("developer".to_string()));
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn shorthand_at_plus_with_save() {
    let processed = pp(&["equip", "@claude", "+developer", "-s", "dev*", "legal/*"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        equip::cli::Command::Equip {
            agent,
            kit,
            save,
            patterns,
            ..
        } => {
            assert_eq!(agent, Some(vec!["claude".to_string()]));
            assert_eq!(kit, Some("developer".to_string()));
            assert!(save);
            assert_eq!(patterns, vec!["dev*".to_string(), "legal/*".to_string()]);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn shorthand_global_flags_preserved() {
    let processed = pp(&["equip", "-n", "--verbose", "@claude"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    assert!(cli.dry_run);
    assert!(cli.verbose);
    match cli.command {
        equip::cli::Command::Equip { agent, .. } => {
            assert_eq!(agent, Some(vec!["claude".to_string()]));
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn save_is_bool_flag() {
    let processed = pp(&["equip", "_equip", "-s", "-k", "mykit", "dev*"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        equip::cli::Command::Equip { save, kit, .. } => {
            assert!(save);
            assert_eq!(kit, Some("mykit".to_string()));
        }
        _ => panic!("expected Equip"),
    }
}
