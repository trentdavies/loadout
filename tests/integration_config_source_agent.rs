use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: create a temp dir and set XDG env vars pointing to it.
/// Returns (temp_dir_guard, config_path, data_dir).
fn setup_env() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("config/equip");
    let data_dir = tmp.path().join("data/equip");
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    (tmp, config_path, data_dir)
}

/// Helper: path to test fixtures.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ─── Config ─────────────────────────────────────────────────────────────

#[test]
fn config_load_default_when_missing() {
    let (_tmp, config_path, _) = setup_env();
    let config = equip::config::load_from(&config_path).unwrap();
    assert!(config.source.is_empty());
    assert!(config.agent.is_empty());
}

#[test]
fn config_save_and_reload() {
    let (_tmp, config_path, _) = setup_env();
    let mut config = equip::config::Config::default();
    config.source.push(equip::config::SourceConfig {
        id: "test".to_string(),
        url: "/tmp/test".to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
        residence: equip::config::SourceResidence::External,
    });
    config.agent.push(equip::config::AgentConfig {
        id: "my-claude".to_string(),
        agent_type: "claude".to_string(),
        path: PathBuf::from("/tmp/targets/claude"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    });
    equip::config::save_to(&config, &config_path).unwrap();

    let reloaded = equip::config::load_from(&config_path).unwrap();
    assert_eq!(reloaded.source.len(), 1);
    assert_eq!(reloaded.source[0].id, "test");
    assert_eq!(reloaded.agent.len(), 1);
    assert_eq!(reloaded.agent[0].agent_type, "claude");
}

#[test]
fn config_roundtrip_with_bundles_and_adapters() {
    let (_tmp, config_path, _) = setup_env();
    let mut config = equip::config::Config::default();
    config.kit.insert(
        "dev".to_string(),
        equip::config::KitConfig {
            skills: vec!["plugin/skill-a".to_string(), "plugin/skill-b".to_string()],
        },
    );
    config.adapter.insert(
        "custom-agent".to_string(),
        equip::config::AdapterConfig {
            skill_dir: "prompts/{name}".to_string(),
            skill_file: "SKILL.md".to_string(),
            format: "agentskills".to_string(),
            copy_dirs: vec!["scripts".to_string()],
        },
    );
    equip::config::save_to(&config, &config_path).unwrap();

    let reloaded = equip::config::load_from(&config_path).unwrap();
    assert_eq!(reloaded.kit.len(), 1);
    assert_eq!(reloaded.kit["dev"].skills.len(), 2);
    assert_eq!(reloaded.adapter.len(), 1);
    assert_eq!(reloaded.adapter["custom-agent"].skill_dir, "prompts/{name}");
}

// ─── Source Detection ───────────────────────────────────────────────────

#[test]
fn detect_single_skill_file() {
    let path = fixtures_dir().join("single-skill/SKILL.md");
    let result = equip::source::detect::detect(&path);
    assert!(result.is_ok());
    match result.unwrap() {
        equip::source::detect::SourceStructure::SingleFile { skill_name } => {
            assert!(!skill_name.is_empty());
        }
        other => panic!("expected SingleFile, got {:?}", other),
    }
}

#[test]
fn detect_plugin_source() {
    let path = fixtures_dir().join("plugin-source");
    let result = equip::source::detect::detect(&path).unwrap();
    match result {
        equip::source::detect::SourceStructure::SinglePlugin => {}
        other => panic!("expected SinglePlugin, got {:?}", other),
    }
}

#[test]
fn detect_flat_skills() {
    let path = fixtures_dir().join("flat-skills");
    let result = equip::source::detect::detect(&path).unwrap();
    match result {
        equip::source::detect::SourceStructure::FlatSkills => {}
        other => panic!("expected FlatSkills, got {:?}", other),
    }
}

#[test]
fn detect_repo_root_with_skills_subdir() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::write(tmp.path().join("README.md"), "# repo").unwrap();
    let skill_dir = tmp.path().join("skills").join("pptx");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: pptx\ndescription: deck skill\n---\n",
    )
    .unwrap();

    let result = equip::source::detect::detect(tmp.path()).unwrap();
    match result {
        equip::source::detect::SourceStructure::FlatSkills => {}
        other => panic!("expected FlatSkills, got {:?}", other),
    }
}

#[test]
fn detect_full_source() {
    let path = fixtures_dir().join("full-source");
    let result = equip::source::detect::detect(&path).unwrap();
    match result {
        equip::source::detect::SourceStructure::Marketplace => {}
        other => panic!("expected Marketplace, got {:?}", other),
    }
}

#[test]
fn detect_invalid_no_frontmatter() {
    let path = fixtures_dir().join("invalid/no-frontmatter/SKILL.md");
    let result = equip::source::detect::detect(&path);
    assert!(result.is_err(), "no-frontmatter file should fail detection");
}

#[test]
fn frontmatter_parsing() {
    let path = fixtures_dir().join("flat-skills/explore/SKILL.md");
    assert!(equip::source::detect::has_skill_frontmatter(&path));
    assert_eq!(
        equip::source::detect::parse_skill_name(&path),
        Some("explore".to_string())
    );
    assert!(equip::source::detect::parse_skill_description(&path).is_some());
}

#[test]
fn frontmatter_missing_returns_none() {
    let path = fixtures_dir().join("invalid/no-frontmatter/SKILL.md");
    assert!(!equip::source::detect::has_skill_frontmatter(&path));
    assert_eq!(equip::source::detect::parse_skill_name(&path), None);
}

// ─── Source Normalization ───────────────────────────────────────────────

#[test]
fn normalize_flat_skills() {
    let path = fixtures_dir().join("flat-skills");
    let parsed = equip::source::ParsedSource::parse(&path)
        .unwrap()
        .with_source_name("flat");
    let registered = equip::source::normalize::normalize(&parsed).unwrap();
    assert_eq!(registered.id, "flat");
    assert!(!registered.plugins.is_empty());
    let total_skills: usize = registered.plugins.iter().map(|p| p.skills.len()).sum();
    assert!(total_skills > 0, "should discover at least one skill");
}

#[test]
fn normalize_plugin_source() {
    let path = fixtures_dir().join("plugin-source");
    let parsed = equip::source::ParsedSource::parse(&path)
        .unwrap()
        .with_source_name("psrc");
    let registered = equip::source::normalize::normalize(&parsed).unwrap();
    assert_eq!(registered.id, "psrc");
    assert!(registered.plugins.iter().any(|p| p.name == "test-plugin"));
    let plugin = registered
        .plugins
        .iter()
        .find(|p| p.name == "test-plugin")
        .unwrap();
    assert!(
        plugin.skills.len() >= 3,
        "test-plugin should have explore, apply, verify"
    );
}

// ─── Registry ───────────────────────────────────────────────────────────

#[test]
fn registry_save_load_roundtrip() {
    let (_tmp, _, data_dir) = setup_env();
    let mut registry = equip::registry::Registry::default();
    registry.sources.push(equip::registry::RegisteredSource {
        id: "test-src".to_string(),
        display_name: None,
        url: String::new(),
        plugins: vec![equip::registry::RegisteredPlugin {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: None,
            skills: vec![equip::registry::RegisteredSkill {
                name: "my-skill".to_string(),
                description: Some("a skill".to_string()),
                author: None,
                version: None,
                path: PathBuf::from("/tmp/cache/my-skill"),
            }],
            path: PathBuf::from("/tmp/cache/test-plugin"),
        }],
        cache_path: PathBuf::from("/tmp/cache"),
        residence: equip::config::SourceResidence::External,
    });
    equip::registry::save_registry(&registry, &data_dir).unwrap();

    let loaded = equip::registry::load_registry(&data_dir).unwrap();
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].plugins[0].skills[0].name, "my-skill");
}

#[test]
fn registry_find_skill_short_form() {
    let mut registry = equip::registry::Registry::default();
    registry.sources.push(equip::registry::RegisteredSource {
        id: "src".to_string(),
        display_name: None,
        url: String::new(),
        plugins: vec![equip::registry::RegisteredPlugin {
            name: "plug".to_string(),
            version: None,
            description: None,
            skills: vec![equip::registry::RegisteredSkill {
                name: "sk".to_string(),
                description: None,
                author: None,
                version: None,
                path: PathBuf::from("/tmp"),
            }],
            path: PathBuf::from("/tmp"),
        }],
        cache_path: PathBuf::from("/tmp"),
        residence: equip::config::SourceResidence::External,
    });

    let (src, plugin, skill) = registry.find_skill("plug/sk").unwrap();
    assert_eq!(src, "src");
    assert_eq!(plugin, "plug");
    assert_eq!(skill.name, "sk");
}

#[test]
fn registry_find_skill_full_form() {
    let mut registry = equip::registry::Registry::default();
    registry.sources.push(equip::registry::RegisteredSource {
        id: "mysrc".to_string(),
        display_name: None,
        url: String::new(),
        plugins: vec![equip::registry::RegisteredPlugin {
            name: "plug".to_string(),
            version: None,
            description: None,
            skills: vec![equip::registry::RegisteredSkill {
                name: "sk".to_string(),
                description: None,
                author: None,
                version: None,
                path: PathBuf::from("/tmp"),
            }],
            path: PathBuf::from("/tmp"),
        }],
        cache_path: PathBuf::from("/tmp"),
        residence: equip::config::SourceResidence::External,
    });

    let (src, _, _) = registry.find_skill("mysrc:plug/sk").unwrap();
    assert_eq!(src, "mysrc");
}

#[test]
fn registry_find_skill_not_found() {
    let registry = equip::registry::Registry::default();
    assert!(registry.find_skill("nope/nada").is_err());
}

// ─── Agent Adapter ──────────────────────────────────────────────────────

#[test]
fn adapter_resolve_builtin_agents() {
    let adapters = std::collections::BTreeMap::new();
    for agent in &["claude", "codex", "cursor", "gemini", "vscode"] {
        let target = equip::config::AgentConfig {
            id: "t".to_string(),
            agent_type: agent.to_string(),
            path: PathBuf::from("/tmp"),
            scope: "machine".to_string(),
            sync: "auto".to_string(),
            equipped: Vec::new(),
        };
        assert!(
            equip::agent::resolve_adapter(&target, &adapters).is_ok(),
            "built-in agent '{}' should resolve",
            agent
        );
    }
}

#[test]
fn adapter_resolve_unknown_agent_fails() {
    let adapters = std::collections::BTreeMap::new();
    let target = equip::config::AgentConfig {
        id: "t".to_string(),
        agent_type: "unknown-agent".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    };
    assert!(equip::agent::resolve_adapter(&target, &adapters).is_err());
}

#[test]
fn adapter_install_uninstall_skill() {
    let tmp = TempDir::new().unwrap();
    let target_path = tmp.path();

    // Create a mock skill source
    let skill_src = TempDir::new().unwrap();
    fs::write(
        skill_src.path().join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test\n---\n# Test",
    )
    .unwrap();

    let adapters = std::collections::BTreeMap::new();
    let target = equip::config::AgentConfig {
        id: "t".to_string(),
        agent_type: "claude".to_string(),
        path: target_path.to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    };
    let adapter = equip::agent::resolve_adapter(&target, &adapters).unwrap();

    let skill = equip::registry::RegisteredSkill {
        name: "test-skill".to_string(),
        description: Some("A test".to_string()),
        author: None,
        version: None,
        path: skill_src.path().to_path_buf(),
    };

    // Install
    adapter.install_skill(&skill, target_path).unwrap();
    assert!(target_path.join("skills/test-skill/SKILL.md").exists());

    // Verify installed_skills lists it
    let installed = adapter.installed_skills(target_path).unwrap();
    assert!(installed.contains(&"test-skill".to_string()));

    // Uninstall
    adapter.uninstall_skill("test-skill", target_path).unwrap();
    assert!(!target_path.join("skills/test-skill").exists());
}

#[test]
fn adapter_custom_toml() {
    let mut adapters = std::collections::BTreeMap::new();
    adapters.insert(
        "my-agent".to_string(),
        equip::config::AdapterConfig {
            skill_dir: "prompts/{name}".to_string(),
            skill_file: "SKILL.md".to_string(),
            format: "agentskills".to_string(),
            copy_dirs: vec![],
        },
    );

    let target = equip::config::AgentConfig {
        id: "t".to_string(),
        agent_type: "my-agent".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    };
    assert!(equip::agent::resolve_adapter(&target, &adapters).is_ok());
}
