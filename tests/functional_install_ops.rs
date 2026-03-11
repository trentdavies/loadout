use std::fs;
use tempfile::TempDir;

use skittle::config::{BundleConfig, Config, SourceConfig, TargetConfig};
use skittle::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry};
use skittle::target::resolve_adapter;

/// Build a source fixture with a plugin containing 3 skills (skill-a, skill-b,
/// skill-c), a Registry that references it, and a Config with a claude target
/// pointing at a temp directory.
fn setup_test_env() -> (TempDir, TempDir, Registry, Config) {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let skill_names = ["skill-a", "skill-b", "skill-c"];
    let mut skills = Vec::new();
    for name in &skill_names {
        let skill_path = source_dir.path().join(name);
        fs::create_dir_all(&skill_path).unwrap();
        fs::write(
            skill_path.join("SKILL.md"),
            format!(
                "---\nname: {}\ndescription: Test skill {}\n---\n# {}\n",
                name, name, name
            ),
        )
        .unwrap();
        skills.push(RegisteredSkill {
            name: name.to_string(),
            description: Some(format!("Test skill {}", name)),
            author: None,
            version: None,
            path: skill_path,
        });
    }

    let mut registry = Registry::default();
    registry.sources.push(RegisteredSource {
        name: "test-source".to_string(),
        plugins: vec![RegisteredPlugin {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Plugin with three skills".to_string()),
            skills,
            path: source_dir.path().to_path_buf(),
        }],
        cache_path: source_dir.path().to_path_buf(),
    });

    let mut config = Config::default();
    config.source.push(SourceConfig {
        name: "test-source".to_string(),
        url: source_dir.path().to_string_lossy().to_string(),
        source_type: "local".to_string(),
    });
    config.target.push(TargetConfig {
        name: "claude".to_string(),
        agent: "claude".to_string(),
        path: target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });

    (source_dir, target_dir, registry, config)
}

// ─── Install specific skill ─────────────────────────────────────────────

#[test]
fn install_specific_skill() {
    let (_source_dir, target_dir, registry, config) = setup_test_env();
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    let (_src, _plug, skill) = registry.find_skill("test-plugin/skill-a").unwrap();
    adapter.install_skill(skill, target_dir.path()).unwrap();

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed, vec!["skill-a".to_string()]);
}

// ─── Install all skills from plugin ─────────────────────────────────────

#[test]
fn install_all_skills_from_plugin() {
    let (_source_dir, target_dir, registry, config) = setup_test_env();
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    let (_src, plugin) = registry.find_plugin("test-plugin").unwrap();
    for skill in &plugin.skills {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 3);
    assert!(installed.contains(&"skill-a".to_string()));
    assert!(installed.contains(&"skill-b".to_string()));
    assert!(installed.contains(&"skill-c".to_string()));
}

// ─── Install bundle ─────────────────────────────────────────────────────

#[test]
fn install_bundle() {
    let (_source_dir, target_dir, registry, mut config) = setup_test_env();
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    config.bundle.insert(
        "dev".to_string(),
        BundleConfig {
            skills: vec![
                "test-plugin/skill-a".to_string(),
                "test-plugin/skill-b".to_string(),
            ],
        },
    );

    let bundle = &config.bundle["dev"];
    for identity in &bundle.skills {
        let (_src, _plug, skill) = registry.find_skill(identity).unwrap();
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(installed.contains(&"skill-a".to_string()));
    assert!(installed.contains(&"skill-b".to_string()));
    assert!(!installed.contains(&"skill-c".to_string()));
}

// ─── Install to specific target ─────────────────────────────────────────

#[test]
fn install_to_specific_target() {
    let (_source_dir, _target_dir, registry, mut config) = setup_test_env();

    let second_target_dir = TempDir::new().unwrap();
    config.target.push(TargetConfig {
        name: "codex".to_string(),
        agent: "codex".to_string(),
        path: second_target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });

    // Install only to the second target
    let adapter = resolve_adapter(&config.target[1], &config.adapter).unwrap();
    let (_src, plugin) = registry.find_plugin("test-plugin").unwrap();
    for skill in &plugin.skills {
        adapter.install_skill(skill, second_target_dir.path()).unwrap();
    }

    // Second target has all 3 skills
    let installed_second = adapter.installed_skills(second_target_dir.path()).unwrap();
    assert_eq!(installed_second.len(), 3);

    // First target remains empty
    let first_adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();
    let first_target_path = &config.target[0].path;
    let installed_first = first_adapter.installed_skills(first_target_path).unwrap();
    assert!(installed_first.is_empty());
}

// ─── Install nonexistent skill fails ────────────────────────────────────

#[test]
fn install_nonexistent_skill_fails() {
    let (_source_dir, _target_dir, registry, _config) = setup_test_env();

    let result = registry.find_skill("nonexistent/skill");
    assert!(result.is_err());
    assert!(
        result.unwrap_err().to_string().contains("not found"),
        "error should indicate skill was not found"
    );
}

// ─── Install nonexistent plugin fails ───────────────────────────────────

#[test]
fn install_nonexistent_plugin_fails() {
    let (_source_dir, _target_dir, registry, _config) = setup_test_env();

    let result = registry.find_plugin("nonexistent");
    assert!(result.is_none());
}

// ─── Uninstall specific skill ───────────────────────────────────────────

#[test]
fn uninstall_specific_skill() {
    let (_source_dir, target_dir, registry, config) = setup_test_env();
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    // Install all 3 skills
    let (_src, plugin) = registry.find_plugin("test-plugin").unwrap();
    for skill in &plugin.skills {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }
    assert_eq!(adapter.installed_skills(target_dir.path()).unwrap().len(), 3);

    // Uninstall skill-a
    adapter.uninstall_skill("skill-a", target_dir.path()).unwrap();

    let remaining = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(remaining.len(), 2);
    assert!(!remaining.contains(&"skill-a".to_string()));
    assert!(remaining.contains(&"skill-b".to_string()));
    assert!(remaining.contains(&"skill-c".to_string()));
}

// ─── Uninstall bundle ───────────────────────────────────────────────────

#[test]
fn uninstall_bundle() {
    let (_source_dir, target_dir, registry, mut config) = setup_test_env();
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    config.bundle.insert(
        "dev".to_string(),
        BundleConfig {
            skills: vec![
                "test-plugin/skill-a".to_string(),
                "test-plugin/skill-b".to_string(),
            ],
        },
    );

    // Install bundle skills
    let bundle = &config.bundle["dev"];
    for identity in &bundle.skills {
        let (_src, _plug, skill) = registry.find_skill(identity).unwrap();
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }
    assert_eq!(adapter.installed_skills(target_dir.path()).unwrap().len(), 2);

    // Uninstall bundle skills
    for identity in &config.bundle["dev"].skills {
        let (_src, _plug, skill) = registry.find_skill(identity).unwrap();
        adapter.uninstall_skill(&skill.name, target_dir.path()).unwrap();
    }

    let remaining = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(remaining.is_empty());
}
