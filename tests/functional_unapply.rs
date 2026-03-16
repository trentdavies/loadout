use clap::Parser;
use loadout::cli::{Cli, Command, AgentCommand};

#[test]
fn parse_unequip_with_patterns() {
    let cli =
        Cli::try_parse_from(["loadout", "agent", "unequip", "legal/review", "--force"]).unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Unequip {
                patterns, force, ..
            } => {
                assert_eq!(patterns, vec!["legal/review".to_string()]);
                assert!(force);
            }
            _ => panic!("expected Unequip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_unequip_kit() {
    let cli = Cli::try_parse_from([
        "loadout",
        "agent",
        "unequip",
        "--kit",
        "work",
        "--all",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Unequip {
                kit, all, force, ..
            } => {
                assert_eq!(kit, Some("work".to_string()));
                assert!(all);
                assert!(force);
            }
            _ => panic!("expected Unequip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_unequip_multiple_agents() {
    let cli = Cli::try_parse_from([
        "loadout",
        "agent",
        "unequip",
        "legal/review",
        "--agent",
        "claude",
        "--agent",
        "codex",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Unequip { agent, .. } => {
                assert_eq!(
                    agent,
                    Some(vec!["claude".to_string(), "codex".to_string()])
                );
            }
            _ => panic!("expected Unequip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_unequip_multiple_patterns() {
    let cli = Cli::try_parse_from([
        "loadout",
        "agent",
        "unequip",
        "legal/review",
        "sales/pitch",
        "--force",
    ])
    .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Unequip { patterns, .. } => {
                assert_eq!(
                    patterns,
                    vec!["legal/review".to_string(), "sales/pitch".to_string()]
                );
            }
            _ => panic!("expected Unequip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_equip_multiple_agents() {
    let cli = Cli::try_parse_from([
        "loadout", "agent", "equip", "legal/*", "--agent", "claude", "--agent", "codex",
    ])
    .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Equip { agent, all, .. } => {
                assert_eq!(
                    agent,
                    Some(vec!["claude".to_string(), "codex".to_string()])
                );
                assert!(!all);
            }
            _ => panic!("expected Equip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_equip_all() {
    let cli =
        Cli::try_parse_from(["loadout", "agent", "equip", "legal/*", "--all"]).unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Equip { all, .. } => {
                assert!(all);
            }
            _ => panic!("expected Equip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_equip_multiple_patterns() {
    let cli = Cli::try_parse_from([
        "loadout",
        "agent",
        "equip",
        "legal/review",
        "sales/pitch",
    ])
    .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Equip { patterns, .. } => {
                assert_eq!(
                    patterns,
                    vec!["legal/review".to_string(), "sales/pitch".to_string()]
                );
            }
            _ => panic!("expected Equip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn parse_equip_single_agent_still_works() {
    let cli =
        Cli::try_parse_from(["loadout", "agent", "equip", "legal/*", "--agent", "claude"])
            .unwrap();
    match cli.command {
        Command::Agent { command } => match command {
            AgentCommand::Equip { agent, .. } => {
                assert_eq!(agent, Some(vec!["claude".to_string()]));
            }
            _ => panic!("expected Equip"),
        },
        _ => panic!("expected Agent"),
    }
}

#[test]
fn equip_agent_conflicts_with_all() {
    let result = Cli::try_parse_from([
        "loadout",
        "agent",
        "equip",
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
        "loadout",
        "agent",
        "unequip",
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
