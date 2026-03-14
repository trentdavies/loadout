use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Create a skill directory with valid SKILL.md frontmatter.
fn make_skill_fixture(parent: &Path, name: &str) {
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
    let scripts = skill_dir.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("run.sh"), "#!/bin/bash\necho ok").unwrap();
}

/// Recursive directory copy helper.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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

/// Set up an isolated environment with config and data directories.
/// Returns (config_path, data_dir, cache_dir).
fn setup_env(env_dir: &Path) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let config_path = env_dir.join("config/skittle/config.toml");
    let data_dir = env_dir.join("data/skittle");
    let cache_dir = data_dir.join("sources");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();
    (config_path, data_dir, cache_dir)
}

/// Create a fixture source directory with a .claude-plugin/plugin.json and the given skill names.
fn make_source_with_skills(source_dir: &Path, skill_names: &[&str]) {
    let skills_path = source_dir.join("skills");
    for name in skill_names {
        make_skill_fixture(&skills_path, name);
    }
    let cp_dir = source_dir.join(".claude-plugin");
    fs::create_dir_all(&cp_dir).unwrap();
    fs::write(
        cp_dir.join("plugin.json"),
        r#"{"name": "test-plugin", "version": "1.0.0", "description": "Test plugin"}"#,
    )
    .unwrap();
}

/// Add a source to the system: copy to cache, detect, normalize, save to registry and config.
/// Returns the loaded registry after saving.
fn add_source(
    source_dir: &Path,
    source_name: &str,
    cache_dir: &Path,
    data_dir: &Path,
    config_path: &Path,
) -> skittle::registry::Registry {
    let cached = cache_dir.join(source_name);
    copy_dir_recursive(source_dir, &cached).unwrap();

    let structure = skittle::source::detect::detect(&cached).unwrap();
    let registered = skittle::source::normalize::normalize(source_name, &cached, &structure).unwrap();

    let mut registry = skittle::registry::load_registry(data_dir).unwrap();
    registry.sources.push(registered);
    skittle::registry::save_registry(&registry, data_dir).unwrap();

    let mut config = skittle::config::load_from(config_path).unwrap();
    config.source.push(skittle::config::SourceConfig {
        name: source_name.to_string(),
        url: source_dir.display().to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
    });
    skittle::config::save_to(&config, config_path).unwrap();

    registry
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn add_local_source_and_verify_registry() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    // Init config
    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    // Create fixture source
    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["analyze", "transform"]);

    // Parse URL, fetch, detect, normalize, save
    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    )
    .unwrap();
    let default_name = source_url.default_name();
    let cached = cache_dir.join(&default_name);
    skittle::source::fetch::fetch(&source_url, &cached, None).unwrap();

    let structure = skittle::source::detect::detect(&cached).unwrap();
    let registered =
        skittle::source::normalize::normalize(&default_name, &cached, &structure).unwrap();

    assert!(!registered.plugins.is_empty(), "should detect at least one plugin");
    let total_skills: usize = registered.plugins.iter().map(|p| p.skills.len()).sum();
    assert_eq!(total_skills, 2, "should find 2 skills");

    // Save to registry and reload
    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(registered);
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    let loaded = skittle::registry::load_registry(&data_dir).unwrap();
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].name, default_name);
    assert!(!loaded.sources[0].plugins.is_empty());

    let skill_names: Vec<&str> = loaded.sources[0]
        .plugins
        .iter()
        .flat_map(|p| p.skills.iter())
        .map(|s| s.name.as_str())
        .collect();
    assert!(skill_names.contains(&"analyze"));
    assert!(skill_names.contains(&"transform"));
}

#[test]
fn add_source_with_custom_name() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["deploy"]);

    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    )
    .unwrap();

    // Use a custom name instead of the default
    let custom_name = "my-custom-source";
    let cached = cache_dir.join(custom_name);
    skittle::source::fetch::fetch(&source_url, &cached, None).unwrap();

    let structure = skittle::source::detect::detect(&cached).unwrap();
    let registered =
        skittle::source::normalize::normalize(custom_name, &cached, &structure).unwrap();

    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(registered);
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    let loaded = skittle::registry::load_registry(&data_dir).unwrap();
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(
        loaded.sources[0].name, custom_name,
        "registry should use the custom name, not the default"
    );
}

#[test]
fn remove_source_cleans_registry() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["ephemeral"]);

    let registry = add_source(
        source_dir.path(),
        "removable-src",
        &cache_dir,
        &data_dir,
        &config_path,
    );
    assert_eq!(registry.sources.len(), 1, "source should be added");

    // Remove the source from registry
    let mut registry = skittle::registry::load_registry(&data_dir).unwrap();
    registry.sources.retain(|s| s.name != "removable-src");
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    // Also remove from config
    let mut config = skittle::config::load_from(&config_path).unwrap();
    config.source.retain(|s| s.name != "removable-src");
    skittle::config::save_to(&config, &config_path).unwrap();

    // Verify both are empty
    let loaded_registry = skittle::registry::load_registry(&data_dir).unwrap();
    assert!(loaded_registry.sources.is_empty(), "registry should be empty after removal");

    let loaded_config = skittle::config::load_from(&config_path).unwrap();
    assert!(loaded_config.source.is_empty(), "config should be empty after removal");
}

#[test]
fn remove_source_with_installed_skills_detected() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());
    let target_dir = TempDir::new().unwrap();

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["guarded"]);

    let registry = add_source(
        source_dir.path(),
        "guarded-src",
        &cache_dir,
        &data_dir,
        &config_path,
    );

    // Set up a target and install the skill
    let mut config = skittle::config::load_from(&config_path).unwrap();
    config.target.push(skittle::config::TargetConfig {
        name: "test-target".to_string(),
        agent: "claude".to_string(),
        path: target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });
    skittle::config::save_to(&config, &config_path).unwrap();

    let adapter = skittle::target::resolve_adapter(&config.target[0], &BTreeMap::new()).unwrap();

    // Install skills from the registry
    let all_skills = registry.all_skills();
    assert!(!all_skills.is_empty(), "should have skills to install");
    for (_, _, skill) in &all_skills {
        adapter.install_skill(skill, target_dir.path()).unwrap();
    }

    // Verify installed_skills detects them — this is the check the CLI does before removal
    let installed = adapter.installed_skills(target_dir.path()).unwrap();
    assert!(
        !installed.is_empty(),
        "installed_skills should be non-empty when skills are installed"
    );
    assert!(
        installed.contains(&"guarded".to_string()),
        "should find the installed skill by name"
    );
}

#[test]
fn list_sources_returns_all() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    // Add three sources
    let names = ["alpha-src", "beta-src", "gamma-src"];
    for name in &names {
        let source_dir = TempDir::new().unwrap();
        make_source_with_skills(source_dir.path(), &["skill-a"]);
        add_source(source_dir.path(), name, &cache_dir, &data_dir, &config_path);
    }

    let loaded_config = skittle::config::load_from(&config_path).unwrap();
    let loaded_registry = skittle::registry::load_registry(&data_dir).unwrap();

    assert_eq!(
        loaded_config.source.len(),
        3,
        "config should have all 3 sources"
    );
    assert_eq!(
        loaded_registry.sources.len(),
        3,
        "registry should have all 3 sources"
    );
    assert_eq!(
        loaded_config.source.len(),
        loaded_registry.sources.len(),
        "config and registry source counts should match"
    );

    // Verify all names are present
    let config_names: Vec<&str> = loaded_config.source.iter().map(|s| s.name.as_str()).collect();
    let registry_names: Vec<&str> = loaded_registry.sources.iter().map(|s| s.name.as_str()).collect();
    for name in &names {
        assert!(config_names.contains(name), "config missing source '{}'", name);
        assert!(registry_names.contains(name), "registry missing source '{}'", name);
    }
}

#[test]
fn show_source_detail() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["inspect", "audit", "report"]);

    add_source(
        source_dir.path(),
        "detail-src",
        &cache_dir,
        &data_dir,
        &config_path,
    );

    // Look up in config
    let config = skittle::config::load_from(&config_path).unwrap();
    let config_entry = config
        .source
        .iter()
        .find(|s| s.name == "detail-src")
        .expect("source should exist in config");
    assert_eq!(config_entry.name, "detail-src");
    assert_eq!(config_entry.source_type, "local");
    assert_eq!(config_entry.url, source_dir.path().display().to_string());

    // Look up in registry
    let registry = skittle::registry::load_registry(&data_dir).unwrap();
    let reg_entry = registry
        .sources
        .iter()
        .find(|s| s.name == "detail-src")
        .expect("source should exist in registry");
    assert_eq!(reg_entry.name, "detail-src");
    assert_eq!(reg_entry.plugins.len(), 1, "should have 1 plugin");

    let plugin = &reg_entry.plugins[0];
    assert_eq!(plugin.name, "test-plugin");
    assert_eq!(plugin.version, Some("1.0.0".to_string()));
    assert_eq!(plugin.skills.len(), 3, "should have 3 skills");

    let skill_names: Vec<&str> = plugin.skills.iter().map(|s| s.name.as_str()).collect();
    assert!(skill_names.contains(&"inspect"));
    assert!(skill_names.contains(&"audit"));
    assert!(skill_names.contains(&"report"));
}

#[test]
fn update_source_re_detects() {
    let env_dir = TempDir::new().unwrap();
    let (config_path, data_dir, cache_dir) = setup_env(env_dir.path());

    let config = skittle::config::Config::default();
    skittle::config::save_to(&config, &config_path).unwrap();

    // Create initial source with one skill
    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["original"]);

    add_source(
        source_dir.path(),
        "evolving-src",
        &cache_dir,
        &data_dir,
        &config_path,
    );

    // Verify initial state
    let registry = skittle::registry::load_registry(&data_dir).unwrap();
    let initial_skills: usize = registry
        .sources
        .iter()
        .flat_map(|s| s.plugins.iter())
        .map(|p| p.skills.len())
        .sum();
    assert_eq!(initial_skills, 1, "should start with 1 skill");

    // Modify the fixture: add a new skill to the source
    make_skill_fixture(&source_dir.path().join("skills"), "added-later");

    // Re-fetch: clear old cache and copy updated source
    let cached = cache_dir.join("evolving-src");
    if cached.exists() {
        fs::remove_dir_all(&cached).unwrap();
    }
    copy_dir_recursive(source_dir.path(), &cached).unwrap();

    // Re-detect and re-normalize
    let structure = skittle::source::detect::detect(&cached).unwrap();
    let updated =
        skittle::source::normalize::normalize("evolving-src", &cached, &structure).unwrap();

    // Replace the source in the registry
    let mut registry = skittle::registry::load_registry(&data_dir).unwrap();
    registry.sources.retain(|s| s.name != "evolving-src");
    registry.sources.push(updated);
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    // Verify the updated registry
    let loaded = skittle::registry::load_registry(&data_dir).unwrap();
    let updated_skills: Vec<&str> = loaded
        .sources
        .iter()
        .flat_map(|s| s.plugins.iter())
        .flat_map(|p| p.skills.iter())
        .map(|s| s.name.as_str())
        .collect();

    assert_eq!(updated_skills.len(), 2, "should now have 2 skills");
    assert!(
        updated_skills.contains(&"original"),
        "original skill should still be present"
    );
    assert!(
        updated_skills.contains(&"added-later"),
        "newly added skill should be detected"
    );
}

#[test]
fn normalize_with_overrides_uses_custom_names() {
    let env_dir = TempDir::new().unwrap();
    let (_config_path, _data_dir, cache_dir) = setup_env(env_dir.path());

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["original"]);

    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    ).unwrap();
    let cached = cache_dir.join("test-src");
    skittle::source::fetch::fetch(&source_url, &cached, None).unwrap();

    let structure = skittle::source::detect::detect(&cached).unwrap();
    let overrides = skittle::source::normalize::Overrides {
        plugin: Some("custom-plug"),
        skill: None,
    };
    let registered = skittle::source::normalize::normalize_with(
        "test-src", &cached, &structure, &overrides,
    ).unwrap();

    assert_eq!(registered.plugins[0].name, "custom-plug");
}

#[test]
fn normalize_with_overrides_rejects_invalid_kebab() {
    let env_dir = TempDir::new().unwrap();
    let (_config_path, _data_dir, cache_dir) = setup_env(env_dir.path());

    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["original"]);

    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    ).unwrap();
    let cached = cache_dir.join("test-src");
    skittle::source::fetch::fetch(&source_url, &cached, None).unwrap();

    let structure = skittle::source::detect::detect(&cached).unwrap();
    let overrides = skittle::source::normalize::Overrides {
        plugin: Some("NotKebab"),
        skill: None,
    };
    let result = skittle::source::normalize::normalize_with(
        "test-src", &cached, &structure, &overrides,
    );
    assert!(result.is_err());
}

#[test]
fn prompt_confirm_uses_default_non_interactive() {
    // In test harness, stdin is not a TTY, so confirm_or_override returns the default
    let result = skittle::prompt::confirm_or_override("Source", "inferred-name", false);
    assert_eq!(result, "inferred-name");
}

#[test]
fn prompt_select_errors_non_interactive() {
    let options = vec!["alpha".to_string(), "beta".to_string()];
    let result = skittle::prompt::select_from("Source", &options, false);
    assert!(result.is_err(), "select_from should error when not interactive");
}

#[test]
fn fetch_local_symlink_creates_symlink() {
    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["alpha"]);

    let cache_dir = TempDir::new().unwrap();
    let cached = cache_dir.path().join("linked-src");

    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    ).unwrap();

    skittle::source::fetch::fetch_with_mode(&source_url, &cached, None, true).unwrap();

    // Cache path should be a symlink
    assert!(cached.symlink_metadata().unwrap().file_type().is_symlink());
    // And it should resolve to the original source
    assert!(cached.join("skills").exists() || cached.join("SKILL.md").exists());
}

#[test]
fn fetch_local_copy_creates_real_dir() {
    let source_dir = TempDir::new().unwrap();
    make_source_with_skills(source_dir.path(), &["beta"]);

    let cache_dir = TempDir::new().unwrap();
    let cached = cache_dir.path().join("copied-src");

    let source_url = skittle::source::SourceUrl::parse(
        source_dir.path().to_str().unwrap(),
    ).unwrap();

    skittle::source::fetch::fetch_with_mode(&source_url, &cached, None, false).unwrap();

    // Cache path should be a real directory, not a symlink
    assert!(cached.is_dir());
    assert!(!cached.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
fn fetch_local_single_file_always_copies() {
    let source_dir = TempDir::new().unwrap();
    let skill_file = source_dir.path().join("SKILL.md");
    fs::write(&skill_file, "---\nname: x\ndescription: d\n---\n").unwrap();

    let cache_dir = TempDir::new().unwrap();
    let cached = cache_dir.path().join("file-src");

    let source_url = skittle::source::SourceUrl::parse(
        skill_file.to_str().unwrap(),
    ).unwrap();

    // Even with symlink=true, single file should be copied
    skittle::source::fetch::fetch_with_mode(&source_url, &cached, None, true).unwrap();

    assert!(cached.is_dir());
    assert!(cached.join("SKILL.md").exists());
    // The cache dir itself should not be a symlink
    assert!(!cached.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
fn prompt_fetch_mode_returns_symlink_non_interactive() {
    let result = skittle::prompt::prompt_fetch_mode(false);
    assert_eq!(result, "symlink");
}
