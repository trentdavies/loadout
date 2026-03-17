use std::fs;
use std::path::PathBuf;

use clap::Parser;
use tempfile::TempDir;

use equip::cli::args::preprocess;
use equip::cli::{Cli, Command};
use equip::config::{AgentConfig, Config, KitConfig, SourceConfig};
use equip::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn pp(args: &[&str]) -> Vec<String> {
    preprocess(args.iter().map(|s| s.to_string()).collect())
}

fn create_skill(base: &std::path::Path, name: &str) -> RegisteredSkill {
    let skill_dir = base.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {}\ndescription: test\n---\nbody", name),
    )
    .unwrap();
    RegisteredSkill {
        name: name.to_string(),
        description: Some("test".to_string()),
        author: None,
        version: None,
        path: skill_dir,
    }
}

/// Set up a temp environment with config, registry, and agent on disk.
/// Returns (xdg_dir, config_path, source_dir, agent_dir).
/// Sets XDG_DATA_HOME so that data_dir() resolves to xdg_dir/equip.
fn setup_env() -> (TempDir, PathBuf, TempDir, TempDir) {
    let xdg_dir = TempDir::new().unwrap();
    let source_dir = TempDir::new().unwrap();
    let agent_dir = TempDir::new().unwrap();

    // data_dir = XDG_DATA_HOME/equip
    let data_dir = xdg_dir.path().join("equip");
    let internal = data_dir.join(".equip");
    fs::create_dir_all(&internal).unwrap();

    let skills = vec![
        create_skill(source_dir.path(), "skill-a"),
        create_skill(source_dir.path(), "skill-b"),
        create_skill(source_dir.path(), "skill-c"),
    ];

    let registry = Registry {
        sources: vec![RegisteredSource {
            name: "test-source".to_string(),
            url: String::new(),
            plugins: vec![RegisteredPlugin {
                name: "test-plugin".to_string(),
                version: None,
                description: None,
                skills,
                path: source_dir.path().to_path_buf(),
            }],
            cache_path: source_dir.path().to_path_buf(),
        }],
        installed: Default::default(),
    };
    let reg_json = serde_json::to_string_pretty(&registry).unwrap();
    fs::write(internal.join("registry.json"), reg_json).unwrap();

    // Create config inside data_dir
    let mut config = Config::default();
    config.source.push(SourceConfig {
        name: "test-source".to_string(),
        url: source_dir.path().to_string_lossy().to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
    });
    config.agent.push(AgentConfig {
        name: "claude".to_string(),
        agent_type: "claude".to_string(),
        path: agent_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });

    let config_path = data_dir.join("equip.toml");
    equip::config::save_to(&config, &config_path).unwrap();

    // Set XDG_DATA_HOME so data_dir() finds our temp registry
    std::env::set_var("XDG_DATA_HOME", xdg_dir.path());

    (xdg_dir, config_path, source_dir, agent_dir)
}

fn run_cli(args: &[&str], config_path: &str) -> anyhow::Result<()> {
    let mut full_args = vec!["equip", "--config", config_path, "-q"];
    full_args.extend_from_slice(args);
    let processed = pp(&full_args);
    let cli = Cli::try_parse_from(&processed).map_err(|e| anyhow::anyhow!("parse error: {}", e))?;
    equip::cli::run(cli)
}

// ─── Parsing tests: @agent and +kit shorthands ──────────────────────────────

#[test]
fn parse_equip_with_plus_kit() {
    let processed = pp(&["equip", "_equip", "+dev", "skill-a"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        Command::Equip { kit, patterns, .. } => {
            assert_eq!(kit, Some("dev".to_string()));
            assert_eq!(patterns, vec!["skill-a".to_string()]);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_plus_kit_with_save() {
    let processed = pp(&["equip", "_equip", "+dev", "-s", "dev*"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        Command::Equip {
            kit,
            save,
            patterns,
            ..
        } => {
            assert_eq!(kit, Some("dev".to_string()));
            assert!(save);
            assert_eq!(patterns, vec!["dev*".to_string()]);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_equip_at_agent_plus_kit() {
    let processed = pp(&["equip", "@claude", "+dev", "web*"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        Command::Equip {
            agent,
            kit,
            patterns,
            ..
        } => {
            assert_eq!(agent, Some(vec!["claude".to_string()]));
            assert_eq!(kit, Some("dev".to_string()));
            assert_eq!(patterns, vec!["web*".to_string()]);
        }
        _ => panic!("expected Equip"),
    }
}

#[test]
fn parse_unequip_with_plus_kit() {
    let processed = pp(&["equip", "_equip", "+dev", "--remove", "--force"]);
    let cli = Cli::try_parse_from(&processed).unwrap();
    match cli.command {
        Command::Equip {
            kit, force, remove, ..
        } => {
            assert_eq!(kit, Some("dev".to_string()));
            assert!(force);
            assert!(remove);
        }
        _ => panic!("expected Equip"),
    }
}

// ─── Behavioral tests: kit not found ────────────────────────────────────────

#[test]
fn equip_missing_kit_no_save_errors() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    let result = run_cli(
        &["_equip", "+nonexistent", "test-plugin/*", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("kit 'nonexistent' not found; add -s to create 'nonexistent'"),
        "expected kit-not-found-with-tip error, got: {}",
        err
    );
}

#[test]
fn equip_missing_kit_only_no_patterns_errors() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    let result = run_cli(
        &["_equip", "-k", "nonexistent", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("kit 'nonexistent' not found"),
        "expected kit-not-found error, got: {}",
        err
    );
}

#[test]
fn equip_existing_kit_works() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    // Add a kit to the config
    let mut config = equip::config::load_from(&config_path).unwrap();
    config.kit.insert(
        "dev".to_string(),
        KitConfig {
            skills: vec!["test-plugin/skill-a".to_string()],
        },
    );
    equip::config::save_to(&config, &config_path).unwrap();

    let result = run_cli(&["_equip", "+dev", "-f"], config_path.to_str().unwrap());

    assert!(
        result.is_ok(),
        "equip with existing kit should succeed: {:?}",
        result.err()
    );
}

#[test]
fn equip_existing_kit_plus_patterns_works() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    let mut config = equip::config::load_from(&config_path).unwrap();
    config.kit.insert(
        "dev".to_string(),
        KitConfig {
            skills: vec!["test-plugin/skill-a".to_string()],
        },
    );
    equip::config::save_to(&config, &config_path).unwrap();

    let result = run_cli(
        &["_equip", "+dev", "test-plugin/skill-b", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(
        result.is_ok(),
        "equip with existing kit + patterns should succeed: {:?}",
        result.err()
    );
}

#[test]
fn equip_missing_kit_with_save_creates_kit() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    // --save with missing kit and --force (non-interactive) should create the kit
    let result = run_cli(
        &["_equip", "+newkit", "-s", "test-plugin/*", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(
        result.is_ok(),
        "equip with -s and missing kit should create it: {:?}",
        result.err()
    );

    // Verify the kit was created in config
    let config = equip::config::load_from(&config_path).unwrap();
    assert!(
        config.kit.contains_key("newkit"),
        "kit 'newkit' should exist after --save"
    );
    assert!(
        !config.kit["newkit"].skills.is_empty(),
        "kit should have skills"
    );
}

#[test]
fn equip_existing_kit_with_save_updates_kit() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    // Create kit with one skill
    let mut config = equip::config::load_from(&config_path).unwrap();
    config.kit.insert(
        "dev".to_string(),
        KitConfig {
            skills: vec!["test-plugin/skill-a".to_string()],
        },
    );
    equip::config::save_to(&config, &config_path).unwrap();

    // Equip with broader pattern + --save --force
    let result = run_cli(
        &["_equip", "+dev", "-s", "test-plugin/*", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(
        result.is_ok(),
        "equip with -s and existing kit should update it: {:?}",
        result.err()
    );

    // Verify the kit was updated
    let config = equip::config::load_from(&config_path).unwrap();
    assert!(
        config.kit["dev"].skills.len() > 1,
        "kit should have more skills after update"
    );
}

#[test]
fn unequip_missing_kit_errors() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    let result = run_cli(
        &["_equip", "--remove", "+nonexistent", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("kit 'nonexistent' not found"),
        "expected kit-not-found error, got: {}",
        err
    );
}

#[test]
fn unequip_missing_kit_with_patterns_still_errors() {
    let (_xdg_dir, config_path, _source_dir, _agent_dir) = setup_env();

    // Even with patterns, unequip should error on missing kit
    let result = run_cli(
        &["_equip", "--remove", "+nonexistent", "test-plugin/*", "-f"],
        config_path.to_str().unwrap(),
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("kit 'nonexistent' not found"),
        "expected kit-not-found error, got: {}",
        err
    );
}
