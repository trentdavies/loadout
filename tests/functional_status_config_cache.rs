use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use skittle::config::{
    AdapterConfig, BundleConfig, Config, SourceConfig, TargetConfig, load_from, save_to,
};
use skittle::registry::{
    RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry, load_registry, save_registry,
};
use skittle::target::resolve_adapter;

// ─── Helpers ─────────────────────────────────────────────────────────────

fn make_source(name: &str, url: &str) -> SourceConfig {
    SourceConfig {
        name: name.to_string(),
        url: url.to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
    }
}

fn make_target(name: &str, agent: &str, path: PathBuf) -> TargetConfig {
    TargetConfig {
        name: name.to_string(),
        agent: agent.to_string(),
        path,
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    }
}

fn make_skill(name: &str, path: PathBuf) -> RegisteredSkill {
    RegisteredSkill {
        name: name.to_string(),
        description: Some(format!("Skill {}", name)),
        author: None,
        version: None,
        path,
    }
}

fn make_plugin(name: &str, skills: Vec<RegisteredSkill>, path: PathBuf) -> RegisteredPlugin {
    RegisteredPlugin {
        name: name.to_string(),
        version: Some("1.0.0".to_string()),
        description: Some(format!("Plugin {}", name)),
        skills,
        path,
    }
}

fn make_registered_source(
    name: &str,
    plugins: Vec<RegisteredPlugin>,
    cache_path: PathBuf,
) -> RegisteredSource {
    RegisteredSource {
        name: name.to_string(),
        plugins,
        cache_path,
    }
}

/// Create a fixture skill directory with a SKILL.md file.
fn create_skill_fixture(parent: &std::path::Path, name: &str) -> PathBuf {
    let skill_dir = parent.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {}\ndescription: Test skill {}\n---\n# {}\n", name, name, name),
    )
    .unwrap();
    skill_dir
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[test]
fn status_with_sources_targets_skills() {
    let tmp = TempDir::new().unwrap();
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Build config with 2 sources and 1 target
    let mut config = Config::default();
    config.source.push(make_source("src-alpha", "/tmp/alpha"));
    config.source.push(make_source("src-beta", "/tmp/beta"));
    config
        .target
        .push(make_target("my-target", "claude", target_dir.path().to_path_buf()));

    assert_eq!(config.source.len(), 2);
    assert_eq!(config.target.len(), 1);

    // Build registry: 2 sources, each with 1 plugin containing 2 skills
    let skill_a1_path = create_skill_fixture(source_dir.path(), "skill-a1");
    let skill_a2_path = create_skill_fixture(source_dir.path(), "skill-a2");
    let skill_b1_path = create_skill_fixture(source_dir.path(), "skill-b1");
    let skill_b2_path = create_skill_fixture(source_dir.path(), "skill-b2");

    let mut registry = Registry::default();
    registry.sources.push(make_registered_source(
        "src-alpha",
        vec![make_plugin(
            "plugin-a",
            vec![
                make_skill("skill-a1", skill_a1_path),
                make_skill("skill-a2", skill_a2_path),
            ],
            source_dir.path().to_path_buf(),
        )],
        source_dir.path().to_path_buf(),
    ));
    registry.sources.push(make_registered_source(
        "src-beta",
        vec![make_plugin(
            "plugin-b",
            vec![
                make_skill("skill-b1", skill_b1_path),
                make_skill("skill-b2", skill_b2_path),
            ],
            source_dir.path().to_path_buf(),
        )],
        source_dir.path().to_path_buf(),
    ));

    // Verify counts
    let source_count = registry.sources.len();
    let plugin_count: usize = registry.sources.iter().map(|s| s.plugins.len()).sum();
    let skill_count = registry.all_skills().len();

    assert_eq!(source_count, 2);
    assert_eq!(plugin_count, 2);
    assert_eq!(skill_count, 4);

    // Install 2 skills and verify installed count
    let adapter = resolve_adapter(&config.target[0], &config.adapter).unwrap();

    let skills_to_install: Vec<&RegisteredSkill> = registry
        .sources
        .iter()
        .flat_map(|s| s.plugins.iter().flat_map(|p| &p.skills))
        .take(2)
        .collect();

    for skill in &skills_to_install {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }

    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert_eq!(installed.len(), 2);

    // Save/reload the registry to confirm persistence
    save_registry(&registry, tmp.path()).unwrap();
    let loaded = load_registry(tmp.path()).unwrap();
    assert_eq!(loaded.sources.len(), 2);
    assert_eq!(loaded.all_skills().len(), 4);
}

#[test]
fn status_with_empty_config() {
    let config = Config::default();
    let registry = Registry::default();

    assert_eq!(config.source.len(), 0);
    assert_eq!(config.target.len(), 0);
    assert!(config.adapter.is_empty());
    assert!(config.bundle.is_empty());

    assert_eq!(registry.sources.len(), 0);

    let plugin_count: usize = registry.sources.iter().map(|s| s.plugins.len()).sum();
    let skill_count = registry.all_skills().len();

    assert_eq!(plugin_count, 0);
    assert_eq!(skill_count, 0);
}

#[test]
fn config_show_returns_content() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();
    config.source.push(make_source("my-source", "/opt/skills"));
    config
        .target
        .push(make_target("my-target", "claude", PathBuf::from("/home/agent")));

    save_to(&config, &config_path).unwrap();

    let content = fs::read_to_string(&config_path).unwrap();

    assert!(content.contains("name = \"my-source\""), "should contain source name");
    assert!(content.contains("url = \"/opt/skills\""), "should contain source url");
    assert!(content.contains("name = \"my-target\""), "should contain target name");
    assert!(content.contains("agent = \"claude\""), "should contain agent type");
    assert!(
        content.contains("[[source]]"),
        "should contain TOML source section header"
    );
    assert!(
        content.contains("[[target]]"),
        "should contain TOML target section header"
    );
}

#[test]
fn cache_clean_removes_sources() {
    let cache_dir = TempDir::new().unwrap();
    let cache_path = cache_dir.path().join("sources");
    fs::create_dir_all(&cache_path).unwrap();

    // Simulate cached source directories with files
    for name in &["cached-src-1", "cached-src-2", "cached-src-3"] {
        let src_dir = cache_path.join(name);
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("manifest.json"), "{}").unwrap();
        fs::write(src_dir.join("data.bin"), "binary content").unwrap();
    }

    // Verify files exist
    let entries_before: Vec<_> = fs::read_dir(&cache_path)
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(entries_before.len(), 3);

    // Clean: remove all contents and recreate
    fs::remove_dir_all(&cache_path).unwrap();
    fs::create_dir_all(&cache_path).unwrap();

    // Verify directory is empty
    let entries_after: Vec<_> = fs::read_dir(&cache_path)
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(entries_after.len(), 0);
    assert!(cache_path.is_dir(), "cache directory should still exist");
}

#[test]
fn config_roundtrip_with_all_sections() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut config = Config::default();

    // Sources
    config.source.push(make_source("local-skills", "/home/skills"));
    config.source.push(make_source("remote-skills", "https://github.com/org/repo"));

    // Targets
    config
        .target
        .push(make_target("claude-dev", "claude", PathBuf::from("/home/claude")));
    config
        .target
        .push(make_target("cursor-work", "cursor", PathBuf::from("/home/cursor")));

    // Adapters
    config.adapter.insert(
        "my-agent".to_string(),
        AdapterConfig {
            skill_dir: "prompts/{name}".to_string(),
            skill_file: "SKILL.md".to_string(),
            format: "agentskills".to_string(),
            copy_dirs: vec!["scripts".to_string(), "assets".to_string()],
        },
    );

    // Bundles
    let mut dev_bundle = BundleConfig::default();
    dev_bundle.skills.push("plugin-a/skill-1".to_string());
    dev_bundle.skills.push("plugin-a/skill-2".to_string());
    dev_bundle.skills.push("plugin-b/skill-3".to_string());
    config.bundle.insert("dev".to_string(), dev_bundle);

    let mut prod_bundle = BundleConfig::default();
    prod_bundle.skills.push("plugin-c/deploy".to_string());
    config.bundle.insert("prod".to_string(), prod_bundle);

    // Save and reload
    save_to(&config, &config_path).unwrap();
    let loaded = load_from(&config_path).unwrap();

    // Sources roundtrip
    assert_eq!(loaded.source.len(), 2);
    assert_eq!(loaded.source[0].name, "local-skills");
    assert_eq!(loaded.source[0].url, "/home/skills");
    assert_eq!(loaded.source[0].source_type, "local");
    assert_eq!(loaded.source[1].name, "remote-skills");
    assert_eq!(loaded.source[1].url, "https://github.com/org/repo");

    // Targets roundtrip
    assert_eq!(loaded.target.len(), 2);
    assert_eq!(loaded.target[0].name, "claude-dev");
    assert_eq!(loaded.target[0].agent, "claude");
    assert_eq!(loaded.target[0].path, PathBuf::from("/home/claude"));
    assert_eq!(loaded.target[0].scope, "machine");
    assert_eq!(loaded.target[0].sync, "auto");
    assert_eq!(loaded.target[1].name, "cursor-work");
    assert_eq!(loaded.target[1].agent, "cursor");
    assert_eq!(loaded.target[1].path, PathBuf::from("/home/cursor"));

    // Adapters roundtrip
    assert_eq!(loaded.adapter.len(), 1);
    assert!(loaded.adapter.contains_key("my-agent"));
    let adapter = &loaded.adapter["my-agent"];
    assert_eq!(adapter.skill_dir, "prompts/{name}");
    assert_eq!(adapter.skill_file, "SKILL.md");
    assert_eq!(adapter.format, "agentskills");
    assert_eq!(adapter.copy_dirs, vec!["scripts", "assets"]);

    // Bundles roundtrip
    assert_eq!(loaded.bundle.len(), 2);
    assert!(loaded.bundle.contains_key("dev"));
    assert!(loaded.bundle.contains_key("prod"));
    assert_eq!(loaded.bundle["dev"].skills.len(), 3);
    assert_eq!(loaded.bundle["dev"].skills[0], "plugin-a/skill-1");
    assert_eq!(loaded.bundle["dev"].skills[1], "plugin-a/skill-2");
    assert_eq!(loaded.bundle["dev"].skills[2], "plugin-b/skill-3");
    assert_eq!(loaded.bundle["prod"].skills.len(), 1);
    assert_eq!(loaded.bundle["prod"].skills[0], "plugin-c/deploy");
}
