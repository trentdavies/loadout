use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn make_config_with_bundle(bundle_name: &str, skills: &[&str]) -> skittle::config::Config {
    let mut config = skittle::config::Config::default();
    config.bundle.insert(
        bundle_name.to_string(),
        skittle::config::BundleConfig {
            skills: skills.iter().map(|s| s.to_string()).collect(),
        },
    );
    config
}

fn make_registry_with_skills(skill_names: &[&str], source_dir: &std::path::Path) -> skittle::registry::Registry {
    let skills: Vec<skittle::registry::RegisteredSkill> = skill_names
        .iter()
        .map(|name| {
            let skill_dir = source_dir.join(name);
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("---\nname: {}\ndescription: Test {}\n---\n# {}\n", name, name, name),
            )
            .unwrap();
            skittle::registry::RegisteredSkill {
                name: name.to_string(),
                description: Some(format!("Test {}", name)),
                author: None,
                version: None,
                path: skill_dir,
            }
        })
        .collect();

    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(skittle::registry::RegisteredSource {
        name: "src".to_string(),
        plugins: vec![skittle::registry::RegisteredPlugin {
            name: "plug".to_string(),
            version: None,
            description: None,
            skills,
            path: source_dir.to_path_buf(),
        }],
        cache_path: source_dir.to_path_buf(),
    });
    registry
}

fn make_adapter() -> skittle::target::Adapter {
    let target = skittle::config::TargetConfig {
        name: "test".to_string(),
        agent: "claude".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    skittle::target::resolve_adapter(&target, &BTreeMap::new()).unwrap()
}

// ─── Bundle Config CRUD ─────────────────────────────────────────────────

#[test]
fn bundle_create_in_config() {
    let mut config = skittle::config::Config::default();
    assert!(!config.bundle.contains_key("dev"));

    config.bundle.insert(
        "dev".to_string(),
        skittle::config::BundleConfig { skills: vec![] },
    );
    assert!(config.bundle.contains_key("dev"));
    assert!(config.bundle["dev"].skills.is_empty());
}

#[test]
fn bundle_delete_from_config() {
    let mut config = make_config_with_bundle("dev", &["plug/sk1"]);
    assert!(config.bundle.contains_key("dev"));

    config.bundle.remove("dev");
    assert!(!config.bundle.contains_key("dev"));
}

#[test]
fn bundle_add_skills() {
    let mut config = make_config_with_bundle("dev", &["plug/sk1"]);

    let bundle = config.bundle.get_mut("dev").unwrap();
    bundle.skills.push("plug/sk2".to_string());
    bundle.skills.push("plug/sk3".to_string());

    assert_eq!(config.bundle["dev"].skills.len(), 3);
}

#[test]
fn bundle_drop_skills() {
    let mut config = make_config_with_bundle("dev", &["plug/sk1", "plug/sk2", "plug/sk3"]);

    let bundle = config.bundle.get_mut("dev").unwrap();
    bundle.skills.retain(|s| s != "plug/sk2");

    assert_eq!(config.bundle["dev"].skills.len(), 2);
    assert!(!config.bundle["dev"].skills.contains(&"plug/sk2".to_string()));
}

#[test]
fn bundle_config_roundtrip() {
    let (_tmp, config_path) = {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("config.toml");
        (tmp, p)
    };

    let config = make_config_with_bundle("production", &["core/explore", "core/apply"]);
    skittle::config::save_to(&config, &config_path).unwrap();

    let loaded = skittle::config::load_from(&config_path).unwrap();
    assert_eq!(loaded.bundle["production"].skills.len(), 2);
    assert!(loaded.bundle["production"].skills.contains(&"core/explore".to_string()));
}

// ─── Active Bundle Tracking ─────────────────────────────────────────────

#[test]
fn active_bundle_set_and_get() {
    let mut registry = skittle::registry::Registry::default();

    registry.set_active_bundle("my-target", "dev");
    assert_eq!(registry.active_bundle("my-target"), Some("dev"));
}

#[test]
fn active_bundle_overwrite() {
    let mut registry = skittle::registry::Registry::default();

    registry.set_active_bundle("t", "bundle-a");
    registry.set_active_bundle("t", "bundle-b");
    assert_eq!(registry.active_bundle("t"), Some("bundle-b"));
}

#[test]
fn active_bundle_clear() {
    let mut registry = skittle::registry::Registry::default();

    registry.set_active_bundle("t", "dev");
    registry.clear_active_bundle("t");
    assert!(registry.active_bundle("t").is_none());
}

#[test]
fn active_bundle_per_target() {
    let mut registry = skittle::registry::Registry::default();

    registry.set_active_bundle("target-a", "bundle-1");
    registry.set_active_bundle("target-b", "bundle-2");

    assert_eq!(registry.active_bundle("target-a"), Some("bundle-1"));
    assert_eq!(registry.active_bundle("target-b"), Some("bundle-2"));
}

#[test]
fn active_bundle_persists_through_save_load() {
    let data_dir = TempDir::new().unwrap();

    let mut registry = skittle::registry::Registry::default();
    registry.set_active_bundle("claude-main", "production");
    skittle::registry::save_registry(&registry, data_dir.path()).unwrap();

    let loaded = skittle::registry::load_registry(data_dir.path()).unwrap();
    assert_eq!(loaded.active_bundle("claude-main"), Some("production"));
}

// ─── Bundle Swap (adapter-level simulation) ─────────────────────────────

#[test]
fn bundle_swap_installs_new_uninstalls_old() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let registry = make_registry_with_skills(&["sk-a", "sk-b", "sk-c", "sk-d"], source_dir.path());
    let adapter = make_adapter();

    // "from" bundle has sk-a, sk-b
    let from_skills: Vec<&skittle::registry::RegisteredSkill> = registry.sources[0].plugins[0]
        .skills
        .iter()
        .filter(|s| s.name == "sk-a" || s.name == "sk-b")
        .collect();

    // "to" bundle has sk-c, sk-d
    let to_skills: Vec<&skittle::registry::RegisteredSkill> = registry.sources[0].plugins[0]
        .skills
        .iter()
        .filter(|s| s.name == "sk-c" || s.name == "sk-d")
        .collect();

    // Install "from" bundle
    for s in &from_skills {
        adapter.install_skill(s, target_dir.path()).unwrap();
    }
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(installed.contains(&"sk-a".to_string()));
    assert!(installed.contains(&"sk-b".to_string()));

    // Swap: uninstall "from", install "to"
    for s in &from_skills {
        adapter.uninstall_skill(&s.name, target_dir.path()).unwrap();
    }
    for s in &to_skills {
        adapter.install_skill(s, target_dir.path()).unwrap();
    }

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"sk-c".to_string()));
    assert!(installed.contains(&"sk-d".to_string()));
    assert!(!installed.contains(&"sk-a".to_string()));
}

#[test]
fn bundle_swap_updates_active_tracking() {
    let mut registry = skittle::registry::Registry::default();

    registry.set_active_bundle("target-1", "old-bundle");
    assert_eq!(registry.active_bundle("target-1"), Some("old-bundle"));

    // Swap updates the active bundle
    registry.set_active_bundle("target-1", "new-bundle");
    assert_eq!(registry.active_bundle("target-1"), Some("new-bundle"));
}
