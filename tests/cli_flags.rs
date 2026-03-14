use clap::Parser;
use skittle::cli::Cli;

/// Verify clap parsing of global flags at the Rust level.
/// Full functional coverage is in Docker suite 00.

#[test]
fn parse_help_flag() {
    // --help causes clap to exit, so we check that parsing without it works
    let cli = Cli::try_parse_from(["skittle", "status"]).unwrap();
    assert!(!cli.json);
    assert!(!cli.quiet);
    assert!(!cli.verbose);
    assert!(!cli.dry_run);
}

#[test]
fn parse_dry_run_flag() {
    let cli = Cli::try_parse_from(["skittle", "-n", "apply", "--all"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_dry_run_long_flag() {
    let cli = Cli::try_parse_from(["skittle", "--dry-run", "status"]).unwrap();
    assert!(cli.dry_run);
}

#[test]
fn parse_json_flag() {
    let cli = Cli::try_parse_from(["skittle", "--json", "status"]).unwrap();
    assert!(cli.json);
}

#[test]
fn parse_quiet_flag() {
    let cli = Cli::try_parse_from(["skittle", "-q", "status"]).unwrap();
    assert!(cli.quiet);
}

#[test]
fn parse_verbose_flag() {
    let cli = Cli::try_parse_from(["skittle", "-v", "status"]).unwrap();
    assert!(cli.verbose);
}

#[test]
fn parse_config_override() {
    let cli = Cli::try_parse_from(["skittle", "--config", "/tmp/alt.toml", "status"]).unwrap();
    assert_eq!(cli.config, Some("/tmp/alt.toml".to_string()));
}

#[test]
fn parse_apply_requires_flag() {
    // apply with no flags should parse OK at clap level (error is in run())
    let result = Cli::try_parse_from(["skittle", "apply"]);
    assert!(result.is_ok());
}

#[test]
fn parse_target_add() {
    let cli = Cli::try_parse_from([
        "skittle", "target", "add", "claude", "/tmp/t", "--scope", "repo", "--name", "my-target"
    ]).unwrap();
    match cli.command {
        skittle::cli::Command::Target { command } => {
            match command {
                skittle::cli::TargetCommand::Add { agent, path, scope, sync, name } => {
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
    let cli = Cli::try_parse_from(["skittle", "add", "/tmp/src", "--source", "my-src"]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { url, source, .. } => {
            assert_eq!(url, "/tmp/src");
            assert_eq!(source, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_deprecated_name_flag() {
    // --name still parses (hidden flag) but the handler will bail with deprecation error
    let cli = Cli::try_parse_from(["skittle", "add", "/tmp/src", "--name", "my-src"]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { name, .. } => {
            assert_eq!(name, Some("my-src".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_with_plugin_and_skill_flags() {
    let cli = Cli::try_parse_from([
        "skittle", "add", "/tmp/src", "--source", "s", "--plugin", "p", "--skill", "sk"
    ]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { source, plugin, skill, .. } => {
            assert_eq!(source, Some("s".to_string()));
            assert_eq!(plugin, Some("p".to_string()));
            assert_eq!(skill, Some("sk".to_string()));
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_symlink_flag() {
    let cli = Cli::try_parse_from(["skittle", "add", "/tmp/src", "--symlink"]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { symlink, copy, .. } => {
            assert!(symlink);
            assert!(!copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_copy_flag() {
    let cli = Cli::try_parse_from(["skittle", "add", "/tmp/src", "--copy"]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { symlink, copy, .. } => {
            assert!(!symlink);
            assert!(copy);
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parse_add_symlink_copy_conflict() {
    let result = Cli::try_parse_from(["skittle", "add", "/tmp/src", "--symlink", "--copy"]);
    assert!(result.is_err(), "--symlink and --copy should conflict");
}

#[test]
fn parse_remove_without_name() {
    let cli = Cli::try_parse_from(["skittle", "remove"]).unwrap();
    match cli.command {
        skittle::cli::Command::Remove { name, force } => {
            assert!(name.is_none());
            assert!(!force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_list() {
    let cli = Cli::try_parse_from(["skittle", "list"]).unwrap();
    match cli.command {
        skittle::cli::Command::List { patterns } => {
            assert!(patterns.is_empty());
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_name() {
    let cli = Cli::try_parse_from(["skittle", "list", "test-plugin/explore"]).unwrap();
    match cli.command {
        skittle::cli::Command::List { patterns } => {
            assert_eq!(patterns, vec!["test-plugin/explore".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_with_multiple_patterns() {
    let cli = Cli::try_parse_from(["skittle", "list", "legal/*", "sales/*"]).unwrap();
    match cli.command {
        skittle::cli::Command::List { patterns } => {
            assert_eq!(patterns, vec!["legal/*".to_string(), "sales/*".to_string()]);
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_remove() {
    let cli = Cli::try_parse_from(["skittle", "remove", "my-source", "--force"]).unwrap();
    match cli.command {
        skittle::cli::Command::Remove { name, force } => {
            assert_eq!(name, Some("my-source".to_string()));
            assert!(force);
        }
        _ => panic!("expected Remove"),
    }
}

#[test]
fn parse_update() {
    let cli = Cli::try_parse_from(["skittle", "update", "my-source"]).unwrap();
    match cli.command {
        skittle::cli::Command::Update { name, .. } => {
            assert_eq!(name, Some("my-source".to_string()));
        }
        _ => panic!("expected Update"),
    }
}

#[test]
fn parse_update_all() {
    let cli = Cli::try_parse_from(["skittle", "update"]).unwrap();
    match cli.command {
        skittle::cli::Command::Update { name, .. } => {
            assert!(name.is_none());
        }
        _ => panic!("expected Update"),
    }
}

#[test]
fn parse_init_with_url() {
    let cli = Cli::try_parse_from(["skittle", "init", "https://github.com/org/skills"]).unwrap();
    match cli.command {
        skittle::cli::Command::Init { url } => {
            assert_eq!(url, Some("https://github.com/org/skills".to_string()));
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn parse_init_without_url() {
    let cli = Cli::try_parse_from(["skittle", "init"]).unwrap();
    match cli.command {
        skittle::cli::Command::Init { url } => {
            assert!(url.is_none());
        }
        _ => panic!("expected Init"),
    }
}
