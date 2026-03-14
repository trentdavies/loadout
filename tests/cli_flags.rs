use clap::Parser;
use loadout::cli::Cli;

/// Verify clap parsing of global flags at the Rust level.
/// Full functional coverage is in Docker suite 00.

#[test]
fn parse_help_flag() {
    // --help causes clap to exit, so we check that parsing without it works
    let cli = Cli::try_parse_from(["loadout", "status"]).unwrap();
    assert!(!cli.json);
    assert!(!cli.quiet);
    assert!(!cli.verbose);
    assert!(!cli.dry_run);
}

#[test]
fn parse_dry_run_flag() {
    let cli = Cli::try_parse_from(["loadout", "-n", "apply", "--all"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_dry_run_long_flag() {
    let cli = Cli::try_parse_from(["loadout", "--dry-run", "status"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_json_flag() {
    let cli = Cli::try_parse_from(["loadout", "--json", "status"]).unwrap();
    assert!(cli.json);
}

#[test]
fn parse_quiet_flag() {
    let cli = Cli::try_parse_from(["loadout", "-q", "status"]).unwrap();
    assert!(cli.quiet);
}

#[test]
fn parse_verbose_flag() {
    let cli = Cli::try_parse_from(["loadout", "-v", "status"]).unwrap();
    assert!(cli.verbose);
}

#[test]
fn parse_config_override() {
    let cli = Cli::try_parse_from(["loadout", "--config", "/tmp/alt.toml", "status"]).unwrap();
    assert_eq!(cli.config, Some("/tmp/alt.toml".to_string()));
}

#[test]
fn parse_apply_requires_flag() {
    // apply with no flags should parse OK at clap level (error is in run())
    let result = Cli::try_parse_from(["loadout", "apply"]);
    assert!(result.is_ok());
}

#[test]
fn parse_target_add() {
    let cli = Cli::try_parse_from([
        "loadout",
        "target",
        "add",
        "claude",
        "/tmp/t",
        "--scope",
        "repo",
        "--name",
        "my-target",
    ])
    .unwrap();
    match cli.command {
        loadout::cli::Command::Target { command } => {
            match command {
                loadout::cli::TargetCommand::Add {
                    agent,
                    path,
                    scope,
                    sync,
                    name,
                } => {
                    assert_eq!(agent, "claude");
                    assert_eq!(scope, "repo");
                    assert_eq!(name, Some("my-target".to_string()));
                    assert!(path.is_some());
                    let _ = sync; // just verify it parsed
                }
                _ => panic!("expected Add"),
            }
        }
        _ => panic!("expected Target"),
    }
}

#[test]
fn parse_add() {
    let cli = Cli::try_parse_from(["loadout", "add", "/tmp/src", "--source", "my-src"]).unwrap();
    match cli.command {
        loadout::cli::Command::Add { url, source, .. } => {
            assert_eq!(url, "/tmp/src");
            assert_eq!(source, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_deprecated_name_flag() {
    // --name still parses (hidden flag) but the handler will bail with deprecation error
    let cli = Cli::try_parse_from(["loadout", "add", "/tmp/src", "--name", "my-src"]).unwrap();
    match cli.command {
        loadout::cli::Command::Add { name, .. } => {
            assert_eq!(name, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_with_plugin_and_skill_flags() {
    let cli = Cli::try_parse_from([
        "loadout", "add", "/tmp/src", "--source", "s", "--plugin", "p", "--skill", "sk",
    ])
    .unwrap();
    match cli.command {
        loadout::cli::Command::Add {
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
    let cli = Cli::try_parse_from(["loadout", "add", "/tmp/src", "--symlink"]).unwrap();
    match cli.command {
        loadout::cli::Command::Add { symlink, copy, .. } => {
            assert!(symlink);
            assert!(!copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_copy_flag() {
    let cli = Cli::try_parse_from(["loadout", "add", "/tmp/src", "--copy"]).unwrap();
    match cli.command {
        loadout::cli::Command::Add { symlink, copy, .. } => {
            assert!(!symlink);
            assert!(copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_symlink_copy_conflict() {
    let result = Cli::try_parse_from(["loadout", "add", "/tmp/src", "--symlink", "--copy"]);
    assert!(result.is_err(), "--symlink and --copy should conflict");
}

#[test]
fn parse_remove_without_name() {
    let cli = Cli::try_parse_from(["loadout", "remove"]).unwrap();
    match cli.command {
        loadout::cli::Command::Remove { name, force } => {
            assert!(name.is_none());
            assert!(!force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_list() {
    let cli = Cli::try_parse_from(["loadout", "list"]).unwrap();
    match cli.command {
        loadout::cli::Command::List { patterns, .. } => {
            assert!(patterns.is_empty());
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_name() {
    let cli = Cli::try_parse_from(["loadout", "list", "test-plugin/explore"]).unwrap();
    match cli.command {
        loadout::cli::Command::List { patterns, .. } => {
            assert_eq!(patterns, vec!["test-plugin/explore".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_multiple_patterns() {
    let cli = Cli::try_parse_from(["loadout", "list", "legal/*", "sales/*"]).unwrap();
    match cli.command {
        loadout::cli::Command::List { patterns, .. } => {
            assert_eq!(patterns, vec!["legal/*".to_string(), "sales/*".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_remove() {
    let cli = Cli::try_parse_from(["loadout", "remove", "my-source", "--force"]).unwrap();
    match cli.command {
        loadout::cli::Command::Remove { name, force } => {
            assert_eq!(name, Some("my-source".to_string()));
            assert!(force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_update() {
    let cli = Cli::try_parse_from(["loadout", "update", "my-source"]).unwrap();
    match cli.command {
        loadout::cli::Command::Update { name, .. } => {
            assert_eq!(name, Some("my-source".to_string()));
        }
        _ => panic!("expected Update"),
    }
}

#[test]
fn parse_update_all() {
    let cli = Cli::try_parse_from(["loadout", "update"]).unwrap();
    match cli.command {
        loadout::cli::Command::Update { name, .. } => {
            assert!(name.is_none());
        }
        _ => panic!("expected Update"),
    }
}

#[test]
fn parse_init_with_url() {
    let cli = Cli::try_parse_from(["loadout", "init", "https://github.com/org/skills"]).unwrap();
    match cli.command {
        loadout::cli::Command::Init { url } => {
            assert_eq!(url, Some("https://github.com/org/skills".to_string()));
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn parse_init_without_url() {
    let cli = Cli::try_parse_from(["loadout", "init"]).unwrap();
    match cli.command {
        loadout::cli::Command::Init { url } => {
            assert!(url.is_none());
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn parse_bundle_activate_with_target() {
    let cli = Cli::try_parse_from([
        "loadout",
        "bundle",
        "activate",
        "dev",
        "my-claude",
        "--force",
    ])
    .unwrap();
    match cli.command {
        loadout::cli::Command::Bundle { command } => match command {
            loadout::cli::BundleCommand::Activate {
                name,
                target,
                all,
                force,
            } => {
                assert_eq!(name, "dev");
                assert_eq!(target, Some("my-claude".to_string()));
                assert!(!all);
                assert!(force);
            }
            _ => panic!("expected Activate"),
        },
        _ => panic!("expected Bundle"),
    }
}

#[test]
fn parse_bundle_activate_with_all() {
    let cli = Cli::try_parse_from(["loadout", "bundle", "activate", "dev", "--all"]).unwrap();
    match cli.command {
        loadout::cli::Command::Bundle { command } => match command {
            loadout::cli::BundleCommand::Activate {
                name, all, target, ..
            } => {
                assert_eq!(name, "dev");
                assert!(all);
                assert!(target.is_none());
            }
            _ => panic!("expected Activate"),
        },
        _ => panic!("expected Bundle"),
    }
}

#[test]
fn parse_bundle_deactivate_with_target() {
    let cli = Cli::try_parse_from([
        "loadout",
        "bundle",
        "deactivate",
        "dev",
        "my-claude",
        "--force",
    ])
    .unwrap();
    match cli.command {
        loadout::cli::Command::Bundle { command } => match command {
            loadout::cli::BundleCommand::Deactivate {
                name,
                target,
                force,
                ..
            } => {
                assert_eq!(name, "dev");
                assert_eq!(target, Some("my-claude".to_string()));
                assert!(force);
            }
            _ => panic!("expected Deactivate"),
        },
        _ => panic!("expected Bundle"),
    }
}

#[test]
fn parse_bundle_swap_no_longer_exists() {
    let result = Cli::try_parse_from(["loadout", "bundle", "swap", "a", "b"]);
    assert!(
        result.is_err(),
        "bundle swap should no longer be a valid subcommand"
    );
}

#[test]
fn known_marketplaces_non_empty() {
    assert!(!loadout::marketplace::KNOWN_MARKETPLACES.is_empty());
    for (name, url) in loadout::marketplace::KNOWN_MARKETPLACES {
        assert!(!name.is_empty(), "marketplace name should not be empty");
        assert!(!url.is_empty(), "marketplace URL should not be empty");
        assert!(
            url.starts_with("https://"),
            "marketplace URL should be https: {}",
            url
        );
    }
}

#[test]
fn multi_select_returns_empty_non_interactive() {
    let result = loadout::prompt::multi_select("Pick", &["a", "b"], &[true, true], false);
    assert!(result.is_empty());
}

#[test]
fn detect_agent_targets_returns_vec() {
    // In a tempdir with no agent dirs, should return empty
    let result = loadout::cli::detect_agent_targets();
    // Can't assert empty because the test runner's home may have agents
    // Just verify it returns without error
    let _ = result;
}
