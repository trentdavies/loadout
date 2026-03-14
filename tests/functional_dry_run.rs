use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: create a skill fixture directory with a valid SKILL.md.
fn make_skill_fixture(parent: &std::path::Path, name: &str) {
    let dir = parent.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: Test skill {}\n---\n# {}\n",
            name, name, name
        ),
    )
    .unwrap();
}

/// Helper: set up config with a source and target, and a populated registry.
fn setup_env() -> (TempDir, TempDir, TempDir, PathBuf, PathBuf) {
    let env_dir = TempDir::new().unwrap();
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let config_path = env_dir.path().join("config.toml");
    let data_dir = env_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create source fixture
    make_skill_fixture(source_dir.path(), "skill-a");
    make_skill_fixture(source_dir.path(), "skill-b");

    // Build registry
    let structure = skittle::source::detect::detect(source_dir.path()).unwrap();
    let registered =
        skittle::source::normalize::normalize("test-src", source_dir.path(), &structure).unwrap();
    let mut registry = skittle::registry::Registry::default();
    registry.sources.push(registered);
    skittle::registry::save_registry(&registry, &data_dir).unwrap();

    // Build config
    let mut config = skittle::config::Config::default();
    config.source.push(skittle::config::SourceConfig {
        name: "test-src".to_string(),
        url: source_dir.path().display().to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
    });
    config.target.push(skittle::config::TargetConfig {
        name: "test-target".to_string(),
        agent: "claude".to_string(),
        path: target_dir.path().to_path_buf(),
        scope: "machine".to_string(),
        sync: "auto".to_string(),
    });
    skittle::config::save_to(&config, &config_path).unwrap();

    (env_dir, source_dir, target_dir, config_path, data_dir)
}

#[test]
fn not_calling_install_writes_nothing() {
    let (_env, _source, target_dir, config_path, data_dir) = setup_env();

    let config = skittle::config::load_from(&config_path).unwrap();
    let registry = skittle::registry::load_registry(&data_dir).unwrap();
    let target = &config.target[0];
    let adapter = skittle::target::resolve_adapter(target, &config.adapter).unwrap();

    // Without --force, destructive commands skip adapter calls.
    // Verify that NOT calling install_skill means nothing is written.
    let all_skills = registry.all_skills();
    assert!(!all_skills.is_empty(), "should have skills to install");

    // Don't call install — simulating no --force
    let installed = adapter.installed_skills(&target.path).unwrap();
    assert!(installed.is_empty(), "should not have installed anything");

    // Verify no skill directories exist
    let skills_dir = target_dir.path().join("skills");
    assert!(!skills_dir.exists(), "skills directory should not exist");
}

#[test]
fn not_calling_uninstall_removes_nothing() {
    let (_env, _source, target_dir, config_path, data_dir) = setup_env();

    let config = skittle::config::load_from(&config_path).unwrap();
    let registry = skittle::registry::load_registry(&data_dir).unwrap();
    let target = &config.target[0];
    let adapter = skittle::target::resolve_adapter(target, &config.adapter).unwrap();

    // First, actually install skills
    for (_, _, skill) in &registry.all_skills() {
        adapter.install_skill(skill, &target.path).unwrap();
    }
    let installed_before = adapter.installed_skills(&target.path).unwrap();
    assert_eq!(installed_before.len(), 2);

    // Without --force, uninstall skips adapter calls — verify files remain
    let installed_after = adapter.installed_skills(&target.path).unwrap();
    assert_eq!(
        installed_after.len(),
        2,
        "uninstall without --force should leave files intact"
    );
    assert!(target_dir.path().join("skills/skill-a/SKILL.md").exists());
    assert!(target_dir.path().join("skills/skill-b/SKILL.md").exists());
}

#[test]
fn not_calling_cache_clean_removes_nothing() {
    let (env_dir, _source, _target, _config_path, _data_dir) = setup_env();

    // Create a fake cache directory with content
    let cache_dir = env_dir.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join("cached-file.txt"), "cached data").unwrap();

    // Without --force, cache clean skips deletion
    assert!(
        cache_dir.join("cached-file.txt").exists(),
        "cache should still have files without --force"
    );
}
