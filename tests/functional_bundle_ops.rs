use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use equip::agent::resolve_adapter;
use equip::config::{load_from, save_to, AgentConfig, Config, KitConfig};
use equip::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn make_kit(skills: &[&str]) -> KitConfig {
    KitConfig {
        skills: skills.iter().map(|s| s.to_string()).collect(),
    }
}

fn make_agent(name: &str, path: PathBuf) -> AgentConfig {
    AgentConfig {
        id: name.to_string(),
        agent_type: "claude".to_string(),
        path,
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    }
}

/// Create a skill fixture directory with a SKILL.md file.
fn create_skill_fixture(base: &std::path::Path, name: &str) -> RegisteredSkill {
    let skill_dir = base.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {}\ndescription: desc\n---\nbody", name),
    )
    .unwrap();
    RegisteredSkill {
        name: name.to_string(),
        description: Some("desc".to_string()),
        author: None,
        version: None,
        path: skill_dir,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// Create a bundle with skills, save to disk, reload, and verify the bundle
/// persists with the correct skill count.
#[test]
fn create_bundle_and_add_skills() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .kit
        .insert("dev".to_string(), make_kit(&["p/skill-a", "p/skill-b"]));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(reloaded.kit.contains_key("dev"));
    assert_eq!(reloaded.kit["dev"].skills.len(), 2);
    assert!(reloaded.kit["dev"]
        .skills
        .contains(&"p/skill-a".to_string()));
    assert!(reloaded.kit["dev"]
        .skills
        .contains(&"p/skill-b".to_string()));
}

/// Add a bundle, save, remove it, save again, reload, and verify it is gone.
#[test]
fn delete_bundle() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .kit
        .insert("ephemeral".to_string(), make_kit(&["p/skill-x"]));
    save_to(&config, &config_path).unwrap();

    // Remove and re-save
    let mut config = load_from(&config_path).unwrap();
    assert!(config.kit.contains_key("ephemeral"));
    config.kit.remove("ephemeral");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(!reloaded.kit.contains_key("ephemeral"));
}

/// Create a bundle with 3 skills, remove one via retain, save, reload, and
/// verify only 2 remain.
#[test]
fn drop_skill_from_bundle() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config.kit.insert(
        "trio".to_string(),
        make_kit(&["p/alpha", "p/beta", "p/gamma"]),
    );
    save_to(&config, &config_path).unwrap();

    let mut config = load_from(&config_path).unwrap();
    let bundle = config.kit.get_mut("trio").unwrap();
    bundle.skills.retain(|s| s != "p/beta");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.kit["trio"].skills.len(), 2);
    assert!(reloaded.kit["trio"].skills.contains(&"p/alpha".to_string()));
    assert!(reloaded.kit["trio"].skills.contains(&"p/gamma".to_string()));
    assert!(!reloaded.kit["trio"].skills.contains(&"p/beta".to_string()));
}

/// Create two bundles with different skills backed by real fixture files.
/// Install bundle A's skills to an agent, then swap to bundle B: uninstall A,
/// install B. Verify B's skills are present and A's are not.
#[test]
fn swap_bundle() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create skill fixtures on disk
    let skill_a1 = create_skill_fixture(source_dir.path(), "skill-a1");
    let skill_a2 = create_skill_fixture(source_dir.path(), "skill-a2");
    let skill_b1 = create_skill_fixture(source_dir.path(), "skill-b1");
    let skill_b2 = create_skill_fixture(source_dir.path(), "skill-b2");

    // Build config with two bundles
    let mut config = Config::default();
    config.kit.insert(
        "bundle-a".to_string(),
        make_kit(&["p/skill-a1", "p/skill-a2"]),
    );
    config.kit.insert(
        "bundle-b".to_string(),
        make_kit(&["p/skill-b1", "p/skill-b2"]),
    );
    config
        .agent
        .push(make_agent("tgt", target_dir.path().to_path_buf()));

    // Build registry with all skills
    let mut registry = Registry::default();
    registry.sources.push(RegisteredSource {
        id: "src".to_string(),
        display_name: None,
        url: String::new(),
        plugins: vec![RegisteredPlugin {
            name: "p".to_string(),
            version: None,
            description: None,
            skills: vec![
                skill_a1.clone(),
                skill_a2.clone(),
                skill_b1.clone(),
                skill_b2.clone(),
            ],
            path: source_dir.path().to_path_buf(),
        }],
        cache_path: source_dir.path().to_path_buf(),
        residence: equip::config::SourceResidence::External,
    });

    let adapter = resolve_adapter(&config.agent[0], &config.adapter).unwrap();

    // Install bundle A
    adapter.install_skill(&skill_a1, target_dir.path()).unwrap();
    adapter.install_skill(&skill_a2, target_dir.path()).unwrap();
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"skill-a1".to_string()));
    assert!(installed.contains(&"skill-a2".to_string()));

    // Swap: uninstall A, install B
    adapter
        .uninstall_skill("skill-a1", target_dir.path())
        .unwrap();
    adapter
        .uninstall_skill("skill-a2", target_dir.path())
        .unwrap();
    adapter.install_skill(&skill_b1, target_dir.path()).unwrap();
    adapter.install_skill(&skill_b2, target_dir.path()).unwrap();
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"skill-b1".to_string()));
    assert!(installed.contains(&"skill-b2".to_string()));
    assert!(!installed.contains(&"skill-a1".to_string()));
    assert!(!installed.contains(&"skill-a2".to_string()));
}

/// Insert a bundle named "dup" and verify contains_key detects it. The CLI
/// would use this check to reject creating a duplicate bundle.
#[test]
fn create_duplicate_bundle_detected() {
    let mut config = Config::default();
    config
        .kit
        .insert("dup".to_string(), make_kit(&["p/skill-one"]));

    // The CLI checks this before inserting:
    let already_exists = config.kit.contains_key("dup");
    assert!(already_exists, "duplicate bundle name should be detected");

    // Attempting a second insert would overwrite — the CLI prevents this
    assert_eq!(config.kit.len(), 1);
}
