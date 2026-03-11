use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use skittle::config::{BundleConfig, Config, TargetConfig, load_from, save_to};
use skittle::registry::{
    load_registry, save_registry, RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry,
};
use skittle::target::resolve_adapter;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn make_bundle(skills: &[&str]) -> BundleConfig {
    BundleConfig {
        skills: skills.iter().map(|s| s.to_string()).collect(),
    }
}

fn make_target(name: &str, path: PathBuf) -> TargetConfig {
    TargetConfig {
        name: name.to_string(),
        agent: "claude".to_string(),
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
        .bundle
        .insert("dev".to_string(), make_bundle(&["p/skill-a", "p/skill-b"]));
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(reloaded.bundle.contains_key("dev"));
    assert_eq!(reloaded.bundle["dev"].skills.len(), 2);
    assert!(reloaded.bundle["dev"].skills.contains(&"p/skill-a".to_string()));
    assert!(reloaded.bundle["dev"].skills.contains(&"p/skill-b".to_string()));
}

/// Add a bundle, save, remove it, save again, reload, and verify it is gone.
#[test]
fn delete_bundle() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config
        .bundle
        .insert("ephemeral".to_string(), make_bundle(&["p/skill-x"]));
    save_to(&config, &config_path).unwrap();

    // Remove and re-save
    let mut config = load_from(&config_path).unwrap();
    assert!(config.bundle.contains_key("ephemeral"));
    config.bundle.remove("ephemeral");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert!(!reloaded.bundle.contains_key("ephemeral"));
}

/// Set a bundle as active on a target via the registry, then verify
/// active_bundle() returns Some. The CLI uses this check to reject deletion
/// without --force.
#[test]
fn delete_active_bundle_detected() {
    let data_dir = TempDir::new().unwrap();

    let mut registry = Registry::default();
    registry.set_active_bundle("my-target", "important");
    save_registry(&registry, data_dir.path()).unwrap();

    let loaded = load_registry(data_dir.path()).unwrap();
    assert_eq!(loaded.active_bundle("my-target"), Some("important"));

    // The CLI would check this before allowing deletion:
    let is_active = loaded.active_bundle("my-target").is_some();
    assert!(is_active, "bundle should be detected as active on target");
}

/// Create a bundle with 3 skills, remove one via retain, save, reload, and
/// verify only 2 remain.
#[test]
fn drop_skill_from_bundle() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config.bundle.insert(
        "trio".to_string(),
        make_bundle(&["p/alpha", "p/beta", "p/gamma"]),
    );
    save_to(&config, &config_path).unwrap();

    let mut config = load_from(&config_path).unwrap();
    let bundle = config.bundle.get_mut("trio").unwrap();
    bundle.skills.retain(|s| s != "p/beta");
    save_to(&config, &config_path).unwrap();

    let reloaded = load_from(&config_path).unwrap();
    assert_eq!(reloaded.bundle["trio"].skills.len(), 2);
    assert!(reloaded.bundle["trio"].skills.contains(&"p/alpha".to_string()));
    assert!(reloaded.bundle["trio"].skills.contains(&"p/gamma".to_string()));
    assert!(!reloaded.bundle["trio"].skills.contains(&"p/beta".to_string()));
}

/// Create two bundles with different skills backed by real fixture files.
/// Install bundle A's skills to a target, then swap to bundle B: uninstall A,
/// install B. Verify B's skills are present and A's are not. Update active
/// bundle tracking accordingly.
#[test]
fn swap_bundle() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    // Create skill fixtures on disk
    let skill_a1 = create_skill_fixture(source_dir.path(), "skill-a1");
    let skill_a2 = create_skill_fixture(source_dir.path(), "skill-a2");
    let skill_b1 = create_skill_fixture(source_dir.path(), "skill-b1");
    let skill_b2 = create_skill_fixture(source_dir.path(), "skill-b2");

    // Build config with two bundles
    let mut config = Config::default();
    config.bundle.insert(
        "bundle-a".to_string(),
        make_bundle(&["p/skill-a1", "p/skill-a2"]),
    );
    config.bundle.insert(
        "bundle-b".to_string(),
        make_bundle(&["p/skill-b1", "p/skill-b2"]),
    );
    config
        .target
        .push(make_target("tgt", target_dir.path().to_path_buf()));

    // Build registry with all skills
    let mut registry = Registry::default();
    registry.sources.push(RegisteredSource {
        name: "src".to_string(),
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
    });

    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    // Install bundle A
    adapter.install_skill(&skill_a1, target_dir.path()).unwrap();
    adapter.install_skill(&skill_a2, target_dir.path()).unwrap();
    registry.set_active_bundle("tgt", "bundle-a");

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"skill-a1".to_string()));
    assert!(installed.contains(&"skill-a2".to_string()));

    // Swap: uninstall A, install B
    adapter.uninstall_skill("skill-a1", target_dir.path()).unwrap();
    adapter.uninstall_skill("skill-a2", target_dir.path()).unwrap();
    adapter.install_skill(&skill_b1, target_dir.path()).unwrap();
    adapter.install_skill(&skill_b2, target_dir.path()).unwrap();
    registry.set_active_bundle("tgt", "bundle-b");

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"skill-b1".to_string()));
    assert!(installed.contains(&"skill-b2".to_string()));
    assert!(!installed.contains(&"skill-a1".to_string()));
    assert!(!installed.contains(&"skill-a2".to_string()));

    // Verify active bundle tracking updated
    assert_eq!(registry.active_bundle("tgt"), Some("bundle-b"));

    // Persist and reload to confirm
    save_registry(&registry, data_dir.path()).unwrap();
    let loaded = load_registry(data_dir.path()).unwrap();
    assert_eq!(loaded.active_bundle("tgt"), Some("bundle-b"));
}

/// Insert a bundle named "dup" and verify contains_key detects it. The CLI
/// would use this check to reject creating a duplicate bundle.
#[test]
fn create_duplicate_bundle_detected() {
    let mut config = Config::default();
    config
        .bundle
        .insert("dup".to_string(), make_bundle(&["p/skill-one"]));

    // The CLI checks this before inserting:
    let already_exists = config.bundle.contains_key("dup");
    assert!(already_exists, "duplicate bundle name should be detected");

    // Attempting a second insert would overwrite — the CLI prevents this
    assert_eq!(config.bundle.len(), 1);
}
