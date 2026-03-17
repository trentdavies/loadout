use clap::Parser;
use equip::cli::{Cli, Command};

#[test]
fn parse_unequip_with_patterns() {
    let cli =
        Cli::try_parse_from(["equip", "_equip", "--remove", "legal/review", "--force"]).unwrap();
    match cli.command {
        Command::Equip {
            patterns, force, remove, ..
        } => {
            assert_eq!(patterns, vec!["legal/review".to_string()]);
            assert!(force);
            assert!(remove);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_unequip_kit() {
    let cli = Cli::try_parse_from([
        "equip",
        "_equip",
        "--remove",
        "--kit",
        "work",
        "--all",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Equip {
            kit, all, force, remove, ..
        } => {
            assert_eq!(kit, Some("work".to_string()));
            assert!(all);
            assert!(force);
            assert!(remove);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_unequip_multiple_agents() {
    let cli = Cli::try_parse_from([
        "equip",
        "_equip",
        "--remove",
        "legal/review",
        "--agent",
        "claude",
        "--agent",
        "codex",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Equip { agent, remove, .. } => {
            assert_eq!(
                agent,
                Some(vec!["claude".to_string(), "codex".to_string()])
            );
            assert!(remove);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_unequip_multiple_patterns() {
    let cli = Cli::try_parse_from([
        "equip",
        "_equip",
        "--remove",
        "legal/review",
        "sales/pitch",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Equip { patterns, remove, .. } => {
            assert_eq!(
                patterns,
                vec!["legal/review".to_string(), "sales/pitch".to_string()]
            );
            assert!(remove);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_multiple_agents() {
    let cli = Cli::try_parse_from([
        "equip", "_equip", "legal/*", "--agent", "claude", "--agent", "codex",
    ])
    .unwrap();
    match cli.command {
        Command::Equip { agent, all, .. } => {
            assert_eq!(
                agent,
                Some(vec!["claude".to_string(), "codex".to_string()])
            );
            assert!(!all);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_all() {
    let cli =
        Cli::try_parse_from(["equip", "_equip", "legal/*", "--all"]).unwrap();
    match cli.command {
        Command::Equip { all, .. } => {
            assert!(all);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_multiple_patterns() {
    let cli = Cli::try_parse_from([
        "equip",
        "_equip",
        "legal/review",
        "sales/pitch",
    ])
    .unwrap();
    match cli.command {
        Command::Equip { patterns, .. } => {
            assert_eq!(
                patterns,
                vec!["legal/review".to_string(), "sales/pitch".to_string()]
            );
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_single_agent_still_works() {
    let cli =
        Cli::try_parse_from(["equip", "_equip", "legal/*", "--agent", "claude"])
            .unwrap();
    match cli.command {
        Command::Equip { agent, .. } => {
            assert_eq!(agent, Some(vec!["claude".to_string()]));
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn equip_agent_conflicts_with_all() {
    let result = Cli::try_parse_from([
        "equip",
        "_equip",
        "legal/*",
        "--agent",
        "claude",
        "--all",
    ]);
    assert!(
        result.is_err(),
        "--agent and --all should conflict"
    );
}

#[test]
fn unequip_agent_conflicts_with_all() {
    let result = Cli::try_parse_from([
        "equip",
        "_equip",
        "--remove",
        "foo",
        "--agent",
        "claude",
        "--all",
    ]);
    assert!(
        result.is_err(),
        "--agent and --all should conflict"
    );
}
