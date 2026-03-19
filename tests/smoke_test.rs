use std::fs;
use tempfile::TempDir;

/// Full end-to-end smoke test exercising the core lifecycle:
/// init → add → target add → install --all → status → uninstall → remove
///
/// This test operates at the library/module level (not CLI binary) to stay safe
/// on the host machine. Full CLI-level E2E is covered by Docker suite 11.
#[test]
fn full_lifecycle_smoke_test() {
    // ── Setup isolated environment ──────────────────────────────────────
    let env_dir = TempDir::new().unwrap();
    let config_path = env_dir.path().join("config/equip/config.toml");
    let data_dir = env_dir.path().join("data/equip");
    let cache_dir = data_dir.join("sources");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let target_dir = TempDir::new().unwrap();

    // Create a fixture source with skills
    let source_dir = TempDir::new().unwrap();
    let skills_path = source_dir.path().join("skills");
    for skill_name in &["explore", "apply", "verify"] {
        let skill_dir = skills_path.join(skill_name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: {}\ndescription: Smoke test skill {}\n---\n# {}\n",
                skill_name, skill_name, skill_name
            ),
        )
        .unwrap();
        let scripts = skill_dir.join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("run.sh"), "#!/bin/bash\necho ok").unwrap();
    }
    // Add a .claude-plugin/plugin.json
    let claude_plugin_dir = source_dir.path().join(".claude-plugin");
    fs::create_dir_all(&claude_plugin_dir).unwrap();
    fs::write(
        claude_plugin_dir.join("plugin.json"),
        r#"{"name": "smoke-plugin", "version": "1.0.0", "description": "Smoke test plugin"}"#,
    )
    .unwrap();

    // ── Step 1: Init (create default config) ────────────────────────────
    let config = equip::config::Config::default();
    equip::config::save_to(&config, &config_path).unwrap();
    assert!(config_path.exists(), "config file should exist after init");

    // ── Step 2: Source add ──────────────────────────────────────────────
    // Simulate: copy source to cache, detect, normalize, register
    let cached = cache_dir.join("smoke-src");
    copy_dir_recursive(source_dir.path(), &cached).unwrap();

    let parsed = equip::source::ParsedSource::parse(&cached)
        .unwrap()
        .with_source_name("smoke-src");
    let registered = equip::source::normalize::normalize(&parsed).unwrap();

    assert!(!registered.plugins.is_empty(), "should detect plugin");
    let total_skills: usize = registered.plugins.iter().map(|p| p.skills.len()).sum();
    assert_eq!(total_skills, 3, "should find 3 skills");

    // Save to registry
    let mut registry = equip::registry::Registry::default();
    registry.sources.push(registered);
    equip::registry::save_registry(&registry, &data_dir).unwrap();

    // Update config with source
    let mut config = equip::config::load_from(&config_path).unwrap();
    config.source.push(equip::config::SourceConfig {
        id: "smoke-src".to_string(),
        url: source_dir.path().display().to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
        residence: equip::config::SourceResidence::External,
    });

    // ── Step 3: Agent add ───────────────────────────────────────────────
    config.agent.push(equip::config::AgentConfig {
        id: "smoke-agent".to_string(),
        agent_type: "claude".to_string(),
        path: target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
        equipped: Vec::new(),
    });
    equip::config::save_to(&config, &config_path).unwrap();

    // Verify config roundtrip
    let config = equip::config::load_from(&config_path).unwrap();
    assert_eq!(config.source.len(), 1);
    assert_eq!(config.agent.len(), 1);

    // ── Step 4: Install --all ───────────────────────────────────────────
    let registry = equip::registry::load_registry(&data_dir).unwrap();
    let target = &config.agent[0];
    let adapter = equip::agent::resolve_adapter(target, &config.adapter).unwrap();

    let all_skills = registry.all_skills();
    assert_eq!(all_skills.len(), 3);

    for (_, _, skill) in &all_skills {
        adapter.install_skill(skill, &target.path).unwrap();
    }

    // Verify all installed
    let installed = adapter.installed_skills(&target.path).unwrap();
    assert_eq!(installed.len(), 3);
    assert!(installed.contains(&"explore".to_string()));
    assert!(installed.contains(&"apply".to_string()));
    assert!(installed.contains(&"verify".to_string()));

    // Verify file content
    let skill_md = target.path.join("skills/explore/SKILL.md");
    assert!(skill_md.exists());
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(content.contains("name: explore"));

    // Scripts copied
    assert!(target.path.join("skills/explore/scripts/run.sh").exists());

    // ── Step 5: Status check ────────────────────────────────────────────
    let registry = equip::registry::load_registry(&data_dir).unwrap();
    let source_count = registry.sources.len();
    let plugin_count: usize = registry.sources.iter().map(|s| s.plugins.len()).sum();
    let skill_count: usize = registry
        .sources
        .iter()
        .flat_map(|s| s.plugins.iter())
        .map(|p| p.skills.len())
        .sum();

    assert_eq!(source_count, 1);
    assert_eq!(plugin_count, 1);
    assert_eq!(skill_count, 3);
    assert_eq!(config.agent.len(), 1);

    // ── Step 6: Uninstall ───────────────────────────────────────────────
    for name in &installed {
        adapter.uninstall_skill(name, &target.path).unwrap();
    }
    let installed = adapter.installed_skills(&target.path).unwrap();
    assert!(installed.is_empty(), "all skills should be uninstalled");
    assert!(!target.path.join("skills/explore/SKILL.md").exists());

    // ── Step 7: Cache clean ─────────────────────────────────────────────
    // Remove cached sources
    if cache_dir.is_dir() {
        fs::remove_dir_all(&cache_dir).unwrap();
        fs::create_dir_all(&cache_dir).unwrap();
    }

    // Clear registry
    let empty_registry = equip::registry::Registry::default();
    equip::registry::save_registry(&empty_registry, &data_dir).unwrap();

    // Verify clean state
    let registry = equip::registry::load_registry(&data_dir).unwrap();
    assert!(
        registry.sources.is_empty(),
        "registry should be empty after cache clean"
    );
    assert!(
        fs::read_dir(&cache_dir).unwrap().count() == 0,
        "cache dir should be empty"
    );
}

/// Recursive directory copy helper.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
