use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use tempfile::TempDir;

use equip::cli::args::preprocess;
use equip::cli::Cli;
use equip::config::{Config, SourceConfig, SourceResidence};
use equip::registry::{InstalledSkill, Registry};

fn pp(args: &[&str]) -> Vec<String> {
    preprocess(args.iter().map(|arg| arg.to_string()).collect())
}

fn run_cli(args: &[&str], config_path: &str) -> anyhow::Result<()> {
    let mut full_args = vec!["equip", "--config", config_path, "-q"];
    full_args.extend_from_slice(args);
    let processed = pp(&full_args);
    let cli = Cli::try_parse_from(&processed).map_err(|err| anyhow::anyhow!("{}", err))?;
    equip::cli::run(cli)
}

fn write_marketplace_plugin(base: &Path, plugin_dir_name: &str, skill_name: &str) {
    let marketplace_dir = base.join(".claude-plugin");
    fs::create_dir_all(&marketplace_dir).unwrap();
    fs::write(
        marketplace_dir.join("marketplace.json"),
        format!(
            r#"{{"name":"test-marketplace","owner":{{"name":"test"}},"plugins":[{{"name":"test-plugin","source":"./{}"}}]}}"#,
            plugin_dir_name
        ),
    )
    .unwrap();

    let plugin_dir = base.join(plugin_dir_name);
    let skill_dir = plugin_dir.join("skills").join(skill_name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {}\ndescription: test\n---\nbody\n", skill_name),
    )
    .unwrap();
    let manifest_dir = plugin_dir.join(".claude-plugin");
    fs::create_dir_all(&manifest_dir).unwrap();
    fs::write(
        manifest_dir.join("plugin.json"),
        r#"{"name":"test-plugin","version":"1.0.0"}"#,
    )
    .unwrap();
}

fn setup_env() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let xdg_dir = TempDir::new().unwrap();
    std::env::set_var("XDG_DATA_HOME", xdg_dir.path());

    let data_dir = xdg_dir.path().join("equip");
    let config_path = data_dir.join("equip.toml");
    let cache_path = data_dir.join("external").join("test-source");
    fs::create_dir_all(data_dir.join(".equip")).unwrap();
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();

    write_marketplace_plugin(&cache_path, "old-plugin", "skill-a");

    let parsed = equip::source::ParsedSource::parse(&cache_path)
        .unwrap()
        .with_source_name("test-source")
        .with_url(cache_path.to_string_lossy().to_string());
    let registered = equip::source::normalize::normalize(&parsed).unwrap();

    let mut registry = Registry::default();
    registry.sources.push(registered);
    registry.installed.insert(
        "claude".to_string(),
        std::collections::BTreeMap::from([(
            "skill-a".to_string(),
            InstalledSkill {
                source: "test-source".to_string(),
                plugin: "test-plugin".to_string(),
                skill: "skill-a".to_string(),
                origin: "external/test-source/old-plugin/skills/skill-a".to_string(),
            },
        )]),
    );
    equip::registry::save_registry(&registry, &data_dir).unwrap();

    let mut config = Config::default();
    config.source.push(SourceConfig {
        name: "test-source".to_string(),
        url: cache_path.to_string_lossy().to_string(),
        source_type: "local".to_string(),
        r#ref: None,
        mode: None,
        residence: SourceResidence::External,
    });
    equip::config::save_to(&config, &config_path).unwrap();

    (xdg_dir, config_path, data_dir, cache_path)
}

#[test]
fn reconcile_updates_installed_origin_after_skill_path_move() {
    let (_xdg_dir, config_path, data_dir, cache_path) = setup_env();

    fs::rename(cache_path.join("old-plugin"), cache_path.join("new-plugin")).unwrap();
    fs::write(
        cache_path.join(".claude-plugin").join("marketplace.json"),
        r#"{"name":"test-marketplace","owner":{"name":"test"},"plugins":[{"name":"test-plugin","source":"./new-plugin"}]}"#,
    )
    .unwrap();

    run_cli(
        &["reconcile", "--source", "test-source"],
        &config_path.to_string_lossy(),
    )
    .unwrap();

    let registry = equip::registry::load_registry(&data_dir).unwrap();
    let source = registry
        .sources
        .iter()
        .find(|source| source.name == "test-source")
        .unwrap();
    let skill = source.plugins[0]
        .skills
        .iter()
        .find(|skill| skill.name == "skill-a")
        .unwrap();
    assert!(skill
        .path
        .ends_with("external/test-source/new-plugin/skills/skill-a"));
    assert_eq!(
        registry.installed["claude"]["skill-a"].origin,
        "external/test-source/new-plugin/skills/skill-a"
    );
}
