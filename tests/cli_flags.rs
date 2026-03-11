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
fn parse_install_requires_flag() {
    // install with no flags should parse OK at clap level (error is in run())
    let result = Cli::try_parse_from(["skittle", "install"]);
    assert!(result.is_ok());
}

#[test]
fn parse_source_add_with_name() {
    let cli = Cli::try_parse_from(["skittle", "source", "add", "/tmp/src", "--name", "my-src"]).unwrap();
    match cli.command {
        skittle::cli::Command::Source { command } => {
            match command {
                skittle::cli::SourceCommand::Add { url, name } => {
                    assert_eq!(url, "/tmp/src");
                    assert_eq!(name, Some("my-src".to_string()));
                }
                _ => panic!("expected Add"),
            }
        }
        _ => panic!("expected Source"),
    }
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
