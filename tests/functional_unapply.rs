use clap::Parser;
use loadout::cli::{Cli, Command};

#[test]
fn parse_unapply_skill() {
    let cli =
        Cli::try_parse_from(["loadout", "unapply", "--skill", "legal/review", "--force"]).unwrap();
    match cli.command {
        Command::Unapply { skill, force, .. } => {
            assert_eq!(skill, Some(vec!["legal/review".to_string()]));
            assert!(force);
        }
        _ => panic!("expected Unapply"),
    }
}

#[test]
fn parse_unapply_bundle() {
    let cli = Cli::try_parse_from([
        "loadout",
        "unapply",
        "--bundle",
        "work",
        "--all-targets",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Unapply {
            bundle,
            all_targets,
            ..
        } => {
            assert_eq!(bundle, Some("work".to_string()));
            assert!(all_targets);
        }
        _ => panic!("expected Unapply"),
    }
}

#[test]
fn parse_unapply_multiple_targets() {
    let cli = Cli::try_parse_from([
        "loadout",
        "unapply",
        "--skill",
        "legal/review",
        "--target",
        "claude",
        "--target",
        "codex",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Unapply { target, .. } => {
            assert_eq!(
                target,
                Some(vec!["claude".to_string(), "codex".to_string()])
            );
        }
        _ => panic!("expected Unapply"),
    }
}

#[test]
fn parse_unapply_multiple_skills() {
    let cli = Cli::try_parse_from([
        "loadout",
        "unapply",
        "--skill",
        "legal/review",
        "sales/pitch",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Unapply { skill, .. } => {
            assert_eq!(
                skill,
                Some(vec![
                    "legal/review".to_string(),
                    "sales/pitch".to_string()
                ])
            );
        }
        _ => panic!("expected Unapply"),
    }
}

#[test]
fn parse_apply_multiple_targets() {
    let cli = Cli::try_parse_from([
        "loadout", "apply", "--all", "--target", "claude", "--target", "codex",
    ])
    .unwrap();
    match cli.command {
        Command::Apply {
            target,
            all_targets,
            ..
        } => {
            assert_eq!(
                target,
                Some(vec!["claude".to_string(), "codex".to_string()])
            );
            assert!(!all_targets);
        }
        _ => panic!("expected Apply"),
    }
}

#[test]
fn parse_apply_all_targets() {
    let cli = Cli::try_parse_from(["loadout", "apply", "--all", "--all-targets"]).unwrap();
    match cli.command {
        Command::Apply { all_targets, .. } => {
            assert!(all_targets);
        }
        _ => panic!("expected Apply"),
    }
}

#[test]
fn parse_apply_multiple_skills() {
    let cli = Cli::try_parse_from([
        "loadout",
        "apply",
        "--skill",
        "legal/review",
        "sales/pitch",
    ])
    .unwrap();
    match cli.command {
        Command::Apply { skill, .. } => {
            assert_eq!(
                skill,
                Some(vec![
                    "legal/review".to_string(),
                    "sales/pitch".to_string()
                ])
            );
        }
        _ => panic!("expected Apply"),
    }
}

#[test]
fn parse_apply_single_target_still_works() {
    let cli =
        Cli::try_parse_from(["loadout", "apply", "--all", "--target", "claude"]).unwrap();
    match cli.command {
        Command::Apply { target, .. } => {
            assert_eq!(target, Some(vec!["claude".to_string()]));
        }
        _ => panic!("expected Apply"),
    }
}

#[test]
fn uninstall_still_parses_as_hidden_alias() {
    let cli = Cli::try_parse_from([
        "loadout",
        "uninstall",
        "--skill",
        "legal/review",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Uninstall { skill, force, .. } => {
            assert_eq!(skill, Some("legal/review".to_string()));
            assert!(force);
        }
        _ => panic!("expected Uninstall (hidden alias)"),
    }
}

#[test]
fn apply_target_conflicts_with_all_targets() {
    let result = Cli::try_parse_from([
        "loadout",
        "apply",
        "--all",
        "--target",
        "claude",
        "--all-targets",
    ]);
    assert!(
        result.is_err(),
        "--target and --all-targets should conflict"
    );
}

#[test]
fn unapply_target_conflicts_with_all_targets() {
    let result = Cli::try_parse_from([
        "loadout",
        "unapply",
        "--skill",
        "foo",
        "--target",
        "claude",
        "--all-targets",
    ]);
    assert!(
        result.is_err(),
        "--target and --all-targets should conflict"
    );
}
