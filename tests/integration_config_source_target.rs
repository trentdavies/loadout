use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: create a temp dir and set XDG env vars pointing to it.
/// Returns (temp_dir_guard, config_path, data_dir).
fn setup_env() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("config/skittle");
    let data_dir = tmp.path().join("data/skittle");
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
    let config = skittle::config::load_from(&config_path).unwrap();
    assert!(config.source.is_empty());
    assert!(config.target.is_empty());
}

#[test]
fn config_save_and_reload() {
    let (_tmp, config_path, _) = setup_env();
    let mut config = skittle::config::Config::default();
    config.source.push(skittle::config::SourceConfig {
        name: "test".to_string(),
        url: "/tmp/test".to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
    });
    config.target.push(skittle::config::TargetConfig {
        name: "my-claude".to_string(),
        agent: "claude".to_string(),
        path: PathBuf::from("/tmp/targets/claude"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });
    skittle::config::save_to(&config, &config_path).unwrap();

    let reloaded = skittle::config::load_from(&config_path).unwrap();
    assert_eq!(reloaded.source.len(), 1);
    assert_eq!(reloaded.source[0].name, "test");
    assert_eq!(reloaded.target.len(), 1);
    assert_eq!(reloaded.target[0].agent, "claude");
}

#[test]
fn config_roundtrip_with_bundles_and_adapters() {
    let (_tmp, config_path, _) = setup_env();
    let mut config = skittle::config::Config::default();
    config.bundle.insert("dev".to_string(), skittle::config::BundleConfig {
        skills: vec!["plugin/skill-a".to_string(), "plugin/skill-b".to_string()],
    });
    config.adapter.insert("custom-agent".to_string(), skittle::config::AdapterConfig {
        skill_dir: "prompts/{name}".to_string(),
        skill_file: "SKILL.md".to_string(),
        format: "agentskills".to_string(),
        copy_dirs: vec!["scripts".to_string()],
    });
    skittle::config::save_to(&config, &config_path).unwrap();

    let reloaded = skittle::config::load_from(&config_path).unwrap();
    assert_eq!(reloaded.bundle.len(), 1);
    assert_eq!(reloaded.bundle["dev"].skills.len(), 2);
    assert_eq!(reloaded.adapter.len(), 1);
    assert_eq!(reloaded.adapter["custom-agent"].skill_dir, "prompts/{name}");
}

// ─── Source Detection ───────────────────────────────────────────────────

#[test]
fn detect_single_skill_file() {
    let path = fixtures_dir().join("single-skill/SKILL.md");
    let result = skittle::source::detect::detect(&path);
    assert!(result.is_ok());
    match result.unwrap() {
        skittle::source::detect::SourceStructure::SingleFile { skill_name } => {
            assert!(!skill_name.is_empty());
        }
        other => panic!("expected SingleFile, got {:?}", other),
    }
}

#[test]
fn detect_plugin_source() {
    let path = fixtures_dir().join("plugin-source");
    let result = skittle::source::detect::detect(&path).unwrap();
    match result {
        skittle::source::detect::SourceStructure::SinglePlugin => {}
        other => panic!("expected SinglePlugin, got {:?}", other),
    }
}

#[test]
fn detect_flat_skills() {
    let path = fixtures_dir().join("flat-skills");
    let result = skittle::source::detect::detect(&path).unwrap();
    match result {
        skittle::source::detect::SourceStructure::FlatSkills => {}
        other => panic!("expected FlatSkills, got {:?}", other),
    }
}

#[test]
fn detect_full_source() {
    let path = fixtures_dir().join("full-source");
    let result = skittle::source::detect::detect(&path).unwrap();
    match result {
        skittle::source::detect::SourceStructure::Marketplace => {}
        other => panic!("expected Marketplace, got {:?}", other),
    }
}

#[test]
fn detect_invalid_no_frontmatter() {
    let path = fixtures_dir().join("invalid/no-frontmatter/SKILL.md");
    let result = skittle::source::detect::detect(&path);
    assert!(result.is_err(), "no-frontmatter file should fail detection");
}

#[test]
fn frontmatter_parsing() {
    let path = fixtures_dir().join("flat-skills/explore/SKILL.md");
    assert!(skittle::source::detect::has_skill_frontmatter(&path));
    assert_eq!(
        skittle::source::detect::parse_skill_name(&path),
        Some("explore".to_string())
    );
    assert!(skittle::source::detect::parse_skill_description(&path).is_some());
}

#[test]
fn frontmatter_missing_returns_none() {
    let path = fixtures_dir().join("invalid/no-frontmatter/SKILL.md");
    assert!(!skittle::source::detect::has_skill_frontmatter(&path));
    assert_eq!(skittle::source::detect::parse_skill_name(&path), None);
}

// ─── Source Normalization ───────────────────────────────────────────────

#[test]
fn normalize_flat_skills() {
    let path = fixtures_dir().join("flat-skills");
    let structure = skittle::source::detect::detect(&path).unwrap();
    let registered = skittle::source::normalize::normalize("flat", &path, &structure).unwrap();
    assert_eq!(registered.name, "flat");
    assert!(!registered.plugins.is_empty());
    let total_skills: usize = registered.plugins.iter().map(|p| p.skills.len()).sum();
    assert!(total_skills > 0, "should discover at least one skill");
}

#[test]
fn normalize_plugin_source() {
    let path = fixtures_dir().join("plugin-source");
    let structure = skittle::source::detect::detect(&path).unwrap();
    let registered = skittle::source::normalize::normalize("psrc", &path, &structure).unwrap();
    assert_eq!(registered.name, "psrc");
    assert!(registered.plugins.iter().any(|p| p.name == "test-plugin"));
    let plugin = registered.plugins.iter().find(|p| p.name == "test-plugin").unwrap();
    assert!(plugin.skills.len() >= 3, "test-plugin should have explore, apply, verify");
}

// ─── Registry ───────────────────────────────────────────────────────────

#[test]
fn registry_save_load_roundtrip() {
    let (_tmp, _, data_dir) = setup_env();
    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(skittle::registry::RegisteredSource {
        name: "test-src".to_string(),
        plugins: vec![skittle::registry::RegisteredPlugin {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: None,
            skills: vec![skittle::registry::RegisteredSkill {
                name: "my-skill".to_string(),
                description: Some("a skill".to_string()),
                author: None,
                version: None,
                path: PathBuf::from("/tmp/cache/my-skill"),
            }],
            path: PathBuf::from("/tmp/cache/test-plugin"),
        }],
        cache_path: PathBuf::from("/tmp/cache"),
    });
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    let loaded = skittle::registry::load_registry(&data_dir).unwrap();
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].plugins[0].skills[0].name, "my-skill");
}

#[test]
fn registry_find_skill_short_form() {
    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(skittle::registry::RegisteredSource {
        name: "src".to_string(),
        plugins: vec![skittle::registry::RegisteredPlugin {
            name: "plug".to_string(),
            version: None,
            description: None,
            skills: vec![skittle::registry::RegisteredSkill {
                name: "sk".to_string(),
                description: None,
                author: None,
                version: None,
                path: PathBuf::from("/tmp"),
            }],
            path: PathBuf::from("/tmp"),
        }],
        cache_path: PathBuf::from("/tmp"),
    });

    let (src, plugin, skill) = registry.find_skill("plug/sk").unwrap();
    assert_eq!(src, "src");
    assert_eq!(plugin, "plug");
    assert_eq!(skill.name, "sk");
}

#[test]
fn registry_find_skill_full_form() {
    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(skittle::registry::RegisteredSource {
        name: "mysrc".to_string(),
        plugins: vec![skittle::registry::RegisteredPlugin {
            name: "plug".to_string(),
            version: None,
            description: None,
            skills: vec![skittle::registry::RegisteredSkill {
                name: "sk".to_string(),
                description: None,
                author: None,
                version: None,
                path: PathBuf::from("/tmp"),
            }],
            path: PathBuf::from("/tmp"),
        }],
        cache_path: PathBuf::from("/tmp"),
    });

    let (src, _, _) = registry.find_skill("mysrc:plug/sk").unwrap();
    assert_eq!(src, "mysrc");
}

#[test]
fn registry_find_skill_not_found() {
    let registry = skittle::registry::Registry::default();
    assert!(registry.find_skill("nope/nada").is_err());
}

// ─── Target Adapter ─────────────────────────────────────────────────────

#[test]
fn adapter_resolve_builtin_agents() {
    let adapters = std::collections::BTreeMap::new();
    for agent in &["claude", "codex", "cursor", "gemini", "vscode"] {
        let target = skittle::config::TargetConfig {
            name: "t".to_string(),
            agent: agent.to_string(),
            path: PathBuf::from("/tmp"),
            scope: "machine".to_string(),
            sync: "auto".to_string(),
        };
        assert!(
            skittle::target::resolve_adapter(&target, &adapters).is_ok(),
            "built-in agent '{}' should resolve",
            agent
        );
    }
}

#[test]
fn adapter_resolve_unknown_agent_fails() {
    let adapters = std::collections::BTreeMap::new();
    let target = skittle::config::TargetConfig {
        name: "t".to_string(),
        agent: "unknown-agent".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    assert!(skittle::target::resolve_adapter(&target, &adapters).is_err());
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
    let target = skittle::config::TargetConfig {
        name: "t".to_string(),
        agent: "claude".to_string(),
        path: target_path.to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    let adapter = skittle::target::resolve_adapter(&target, &adapters).unwrap();

    let skill = skittle::registry::RegisteredSkill {
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
    adapters.insert("my-agent".to_string(), skittle::config::AdapterConfig {
        skill_dir: "prompts/{name}".to_string(),
        skill_file: "SKILL.md".to_string(),
        format: "agentskills".to_string(),
        copy_dirs: vec![],
    });

    let target = skittle::config::TargetConfig {
        name: "t".to_string(),
        agent: "my-agent".to_string(),
        path: PathBuf::from("/tmp"),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    };
    assert!(skittle::target::resolve_adapter(&target, &adapters).is_ok());
}
