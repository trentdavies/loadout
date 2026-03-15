use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a mock skill source directory with N skills.
fn create_mock_skills(
    parent: &std::path::Path,
    names: &[&str],
) -> Vec<loadout::registry::RegisteredSkill> {
    let mut skills = Vec::new();
    for name in names {
        let skill_dir = parent.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: {}\ndescription: Test skill {}\n---\n# {}\n",
                name, name, name
            ),
        )
        .unwrap();
        // Add a scripts/ subdirectory to test copy_dirs
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        fs::write(scripts_dir.join("run.sh"), "#!/bin/bash\necho hello\n").unwrap();

        skills.push(loadout::registry::RegisteredSkill {
            name: name.to_string(),
            description: Some(format!("Test skill {}", name)),
            author: None,
            version: None,
            path: skill_dir,
        });
    }
    skills
}

fn make_adapter() -> loadout::agent::Adapter {
    let agent = loadout::config::AgentConfig {
        name: "test".to_string(),
        agent_type: "claude".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    loadout::agent::resolve_adapter(&agent, &BTreeMap::new()).unwrap()
}

// ─── Install ────────────────────────────────────────────────────────────

#[test]
fn install_single_skill() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["my-skill"]);
    let adapter = make_adapter();

    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();

    assert!(target_dir.path().join("skills/my-skill/SKILL.md").exists());
    assert!(target_dir
        .path()
        .join("skills/my-skill/scripts/run.sh")
        .exists());
}

#[test]
fn install_multiple_skills() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["alpha", "beta", "gamma"]);
    let adapter = make_adapter();

    for skill in &skills {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 3);
    assert!(installed.contains(&"alpha".to_string()));
    assert!(installed.contains(&"beta".to_string()));
    assert!(installed.contains(&"gamma".to_string()));
}

#[test]
fn install_idempotent() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["repeat"]);
    let adapter = make_adapter();

    // Install twice — should not error
    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();
    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 1);
    assert!(target_dir.path().join("skills/repeat/SKILL.md").exists());
}

#[test]
fn install_overwrites_existing() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["evolve"]);
    let adapter = make_adapter();

    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();

    // Modify the source skill
    fs::write(
        source_dir.path().join("evolve/SKILL.md"),
        "---\nname: evolve\ndescription: Updated\n---\n# Updated\n",
    )
    .unwrap();

    // Re-install
    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();

    let content = fs::read_to_string(target_dir.path().join("skills/evolve/SKILL.md")).unwrap();
    assert!(content.contains("Updated"));
}

// ─── Uninstall ──────────────────────────────────────────────────────────

#[test]
fn uninstall_single_skill() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["removable"]);
    let adapter = make_adapter();

    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();
    assert!(target_dir.path().join("skills/removable/SKILL.md").exists());

    adapter
        .uninstall_skill("removable", target_dir.path())
        .unwrap();
    assert!(!target_dir.path().join("skills/removable").exists());
}

#[test]
fn uninstall_nonexistent_is_noop() {
    let target_dir = TempDir::new().unwrap();
    let adapter = make_adapter();

    // Should not error
    adapter.uninstall_skill("ghost", target_dir.path()).unwrap();
}

#[test]
fn uninstall_selective_keeps_others() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["keep", "remove"]);
    let adapter = make_adapter();

    for s in &skills {
        adapter.install_skill(s, target_dir.path()).unwrap();
    }

    adapter
        .uninstall_skill("remove", target_dir.path())
        .unwrap();

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed, vec!["keep".to_string()]);
    assert!(target_dir.path().join("skills/keep/SKILL.md").exists());
    assert!(!target_dir.path().join("skills/remove").exists());
}

// ─── installed_skills listing ───────────────────────────────────────────

#[test]
fn installed_skills_empty_target() {
    let target_dir = TempDir::new().unwrap();
    let adapter = make_adapter();
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(installed.is_empty());
}

#[test]
fn installed_skills_ignores_non_skill_dirs() {
    let target_dir = TempDir::new().unwrap();
    let adapter = make_adapter();

    // Create a skills/ dir with a non-skill subdir (no SKILL.md)
    let fake_dir = target_dir.path().join("skills/not-a-skill");
    fs::create_dir_all(&fake_dir).unwrap();
    fs::write(fake_dir.join("README.md"), "not a skill").unwrap();

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(installed.is_empty());
}

// ─── Custom adapter ─────────────────────────────────────────────────────

#[test]
fn custom_adapter_installs_to_custom_path() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let skills = create_mock_skills(source_dir.path(), &["custom-sk"]);

    let mut adapters = BTreeMap::new();
    adapters.insert(
        "my-agent".to_string(),
        loadout::config::AdapterConfig {
            skill_dir: "prompts/{name}".to_string(),
            skill_file: "SKILL.md".to_string(),
            format: "agentskills".to_string(),
            copy_dirs: vec!["scripts".to_string()],
        },
    );

    let target = loadout::config::AgentConfig {
        name: "custom".to_string(),
        agent_type: "my-agent".to_string(),
        path: target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    let adapter = loadout::agent::resolve_adapter(&target, &adapters).unwrap();

    adapter
        .install_skill(&skills[0], target_dir.path())
        .unwrap();

    // Should be under prompts/ not skills/
    assert!(target_dir
        .path()
        .join("prompts/custom-sk/SKILL.md")
        .exists());
    assert!(target_dir
        .path()
        .join("prompts/custom-sk/scripts/run.sh")
        .exists());
}

// ─── Registry + adapter lifecycle ───────────────────────────────────────

#[test]
fn full_install_uninstall_lifecycle() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    let skills = create_mock_skills(source_dir.path(), &["explore", "apply", "verify"]);
    let adapter = make_adapter();

    // Build a registry with these skills
    let mut registry = loadout::registry::Registry::default();
    registry.sources.push(loadout::registry::RegisteredSource {
        name: "test-src".to_string(),
        url: String::new(),
        plugins: vec![loadout::registry::RegisteredPlugin {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test plugin".to_string()),
            skills: skills.clone(),
            path: source_dir.path().to_path_buf(),
        }],
        cache_path: source_dir.path().to_path_buf(),
    });

    // Save and reload registry
    loadout::registry::save_registry(&registry, data_dir.path()).unwrap();
    let loaded = loadout::registry::load_registry(data_dir.path()).unwrap();
    assert_eq!(loaded.sources[0].plugins[0].skills.len(), 3);

    // Install all skills
    for skill in &skills {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 3);

    // Uninstall one
    adapter.uninstall_skill("apply", target_dir.path()).unwrap();
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);
    assert!(!installed.contains(&"apply".to_string()));

    // Uninstall remaining
    adapter
        .uninstall_skill("explore", target_dir.path())
        .unwrap();
    adapter
        .uninstall_skill("verify", target_dir.path())
        .unwrap();
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(installed.is_empty());
}
