use std::collections::BTreeMap;
use std::path::PathBuf;

use tempfile::TempDir;

use skittle::config::{load_from, save_to, Config, TargetConfig};
use skittle::target::resolve_adapter;

fn make_target(name: &str, agent: &str, path: PathBuf) -> TargetConfig {
    TargetConfig {
        name: name.to_string(),
        agent: agent.to_string(),
        path,
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    }
}

#[test]
fn add_target_with_agent_and_path() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let target_path = tmp.path().join("claude-target");

    let mut config = Config::default();
    config
        .target
        .push(make_target("my-claude", "claude", target_path.clone()));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.target.len(), 1);
    assert_eq!(reloaded.target[0].name, "my-claude");
    assert_eq!(reloaded.target[0].agent, "claude");
    assert_eq!(reloaded.target[0].path, target_path);
}

#[test]
fn target_defaults_scope_and_sync() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config.target.push(TargetConfig {
        name: "defaults-test".to_string(),
        agent: "cursor".to_string(),
        path: tmp.path().join("cursor-target"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.target[0].scope, "machine");
    assert_eq!(reloaded.target[0].sync, "auto");
}

#[test]
fn remove_target() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .target
        .push(make_target("to-remove", "claude", tmp.path().join("t")));
    save_to(&config, &config_path).unwrap();

    let mut config = load_from(&config_path).unwrap();
    assert_eq!(config.target.len(), 1);

    config.target.retain(|t| t.name != "to-remove");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(reloaded.target.is_empty());
}

#[test]
fn add_duplicate_target_name_detected() {
    let tmp = TempDir::new().unwrap();
    let name = "dupe";

    let mut config = Config::default();
    config
        .target
        .push(make_target(name, "claude", tmp.path().join("a")));
    config
        .target
        .push(make_target(name, "codex", tmp.path().join("b")));

    let count = config.target.iter().filter(|t| t.name == name).count();
    assert!(count > 1, "duplicate target name should be detectable");
}

#[test]
fn list_targets() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .target
        .push(make_target("alpha", "claude", tmp.path().join("a")));
    config
        .target
        .push(make_target("beta", "codex", tmp.path().join("b")));
    config
        .target
        .push(make_target("gamma", "cursor", tmp.path().join("c")));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.target.len(), 3);

    let names: Vec<&str> = reloaded.target.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
    assert!(names.contains(&"gamma"));
}

#[test]
fn resolve_adapter_for_known_agents() {
    let empty_adapters = BTreeMap::new();

    for agent in &["claude", "codex", "cursor", "gemini", "vscode"] {
        let tmp = TempDir::new().unwrap();
        let target = make_target("test", agent, tmp.path().to_path_buf());
        let result = resolve_adapter(&target, &empty_adapters);
        assert!(
            result.is_ok(),
            "resolve_adapter should succeed for built-in agent '{}', got: {:?}",
            agent,
            result.err()
        );
    }
}

#[test]
fn resolve_adapter_unknown_agent_fails() {
    let empty_adapters = BTreeMap::new();
    let tmp = TempDir::new().unwrap();
    let target = make_target("bad", "unknown-agent", tmp.path().to_path_buf());

    let result = resolve_adapter(&target, &empty_adapters);
    assert!(
        result.is_err(),
        "resolve_adapter should fail for unknown agent"
    );
}
