use std::collections::BTreeMap;
use std::path::PathBuf;

use tempfile::TempDir;

use equip::agent::resolve_adapter;
use equip::config::{load_from, save_to, AgentConfig, Config};

fn make_agent(name: &str, agent_type: &str, path: PathBuf) -> AgentConfig {
    AgentConfig {
        id: name.to_string(),
        agent_type: agent_type.to_string(),
        path,
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    }
}

#[test]
fn add_agent_with_type_and_path() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let agent_path = tmp.path().join("claude-agent");

    let mut config = Config::default();
    config
        .agent
        .push(make_agent("my-claude", "claude", agent_path.clone()));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.agent.len(), 1);
    assert_eq!(reloaded.agent[0].id, "my-claude");
    assert_eq!(reloaded.agent[0].agent_type, "claude");
    assert_eq!(reloaded.agent[0].path, agent_path);
}

#[test]
fn agent_defaults_scope_and_sync() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config.agent.push(AgentConfig {
        id: "defaults-test".to_string(),
        agent_type: "cursor".to_string(),
        path: tmp.path().join("cursor-agent"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    });
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.agent[0].scope, "machine");
    assert_eq!(reloaded.agent[0].sync, "auto");
}

#[test]
fn remove_agent() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .agent
        .push(make_agent("to-remove", "claude", tmp.path().join("t")));
    save_to(&config, &config_path).unwrap();

    let mut config = load_from(&config_path).unwrap();
    assert_eq!(config.agent.len(), 1);

    config.agent.retain(|t| t.id != "to-remove");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(reloaded.agent.is_empty());
}

#[test]
fn add_duplicate_agent_name_detected() {
    let tmp = TempDir::new().unwrap();
    let name = "dupe";

    let mut config = Config::default();
    config
        .agent
        .push(make_agent(name, "claude", tmp.path().join("a")));
    config
        .agent
        .push(make_agent(name, "codex", tmp.path().join("b")));

    let count = config.agent.iter().filter(|t| t.id == name).count();
    assert!(count > 1, "duplicate agent name should be detectable");
}

#[test]
fn list_agents() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .agent
        .push(make_agent("alpha", "claude", tmp.path().join("a")));
    config
        .agent
        .push(make_agent("beta", "codex", tmp.path().join("b")));
    config
        .agent
        .push(make_agent("gamma", "cursor", tmp.path().join("c")));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.agent.len(), 3);

    let names: Vec<&str> = reloaded.agent.iter().map(|t| t.id.as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
    assert!(names.contains(&"gamma"));
}

#[test]
fn resolve_adapter_for_known_agents() {
    let empty_adapters = BTreeMap::new();

    for agent_type in &["claude", "codex", "cursor", "gemini", "vscode"] {
        let tmp = TempDir::new().unwrap();
        let agent = make_agent("test", agent_type, tmp.path().to_path_buf());
        let result = resolve_adapter(&agent, &empty_adapters);
        assert!(
            result.is_ok(),
            "resolve_adapter should succeed for built-in agent '{}', got: {:?}",
            agent_type,
            result.err()
        );
    }
}

#[test]
fn resolve_adapter_unknown_agent_fails() {
    let empty_adapters = BTreeMap::new();
    let tmp = TempDir::new().unwrap();
    let agent = make_agent("bad", "unknown-agent", tmp.path().to_path_buf());

    let result = resolve_adapter(&agent, &empty_adapters);
    assert!(
        result.is_err(),
        "resolve_adapter should fail for unknown agent"
    );
}
