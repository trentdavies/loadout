use std::path::PathBuf;

use equip::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource, Registry};

/// Build a skill with the given name and no description.
fn skill(name: &str) -> RegisteredSkill {
    RegisteredSkill {
        name: name.to_string(),
        description: None,
        author: None,
        version: None,
        path: PathBuf::from("/tmp"),
    }
}

/// Build a plugin with the given name and a set of skills.
fn plugin(name: &str, skills: Vec<RegisteredSkill>) -> RegisteredPlugin {
    RegisteredPlugin {
        name: name.to_string(),
        version: None,
        description: None,
        skills,
        path: PathBuf::from("/tmp"),
    }
}

/// Build a source with the given name and a set of plugins.
fn source(name: &str, plugins: Vec<RegisteredPlugin>) -> RegisteredSource {
    RegisteredSource {
        name: name.to_string(),
        display_name: None,
        url: String::new(),
        plugins,
        cache_path: PathBuf::from("/tmp"),
        residence: equip::config::SourceResidence::External,
    }
}

// ─── Plugin listing ─────────────────────────────────────────────────────

#[test]
fn list_plugins_across_sources() {
    let mut registry = Registry::default();
    registry
        .sources
        .push(source("src-a", vec![plugin("alpha", vec![skill("s1")])]));
    registry
        .sources
        .push(source("src-b", vec![plugin("beta", vec![skill("s2")])]));

    let plugin_count: usize = registry.sources.iter().map(|s| s.plugins.len()).sum();
    assert_eq!(plugin_count, 2);
}

#[test]
fn list_plugins_filtered_by_source() {
    let mut registry = Registry::default();
    registry
        .sources
        .push(source("src-a", vec![plugin("alpha", vec![skill("s1")])]));
    registry
        .sources
        .push(source("src-b", vec![plugin("beta", vec![skill("s2")])]));

    let filtered: Vec<&RegisteredPlugin> = registry
        .sources
        .iter()
        .filter(|s| s.name == "src-a")
        .flat_map(|s| &s.plugins)
        .collect();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "alpha");
}

#[test]
fn show_plugin_detail() {
    let mut registry = Registry::default();
    let mut p = plugin("detail-plugin", vec![skill("sk")]);
    p.version = Some("2.3.1".to_string());
    p.description = Some("A detailed plugin".to_string());
    registry.sources.push(source("src", vec![p]));

    let (src_name, found) = registry.find_plugin("detail-plugin").unwrap();
    assert_eq!(src_name, "src");
    assert_eq!(found.name, "detail-plugin");
    assert_eq!(found.version.as_deref(), Some("2.3.1"));
    assert_eq!(found.description.as_deref(), Some("A detailed plugin"));
}

// ─── Skill listing ──────────────────────────────────────────────────────

#[test]
fn list_skills_across_sources() {
    let mut registry = Registry::default();
    registry.sources.push(source(
        "src-a",
        vec![plugin("p1", vec![skill("a1"), skill("a2")])],
    ));
    registry
        .sources
        .push(source("src-b", vec![plugin("p2", vec![skill("b1")])]));

    let all = registry.all_skills();
    assert_eq!(all.len(), 3);
}

#[test]
fn list_skills_filtered_by_plugin() {
    let mut registry = Registry::default();
    registry.sources.push(source(
        "src",
        vec![
            plugin("keep", vec![skill("k1"), skill("k2")]),
            plugin("skip", vec![skill("s1")]),
        ],
    ));

    let filtered: Vec<_> = registry
        .all_skills()
        .into_iter()
        .filter(|(_, p, _)| p.name == "keep")
        .collect();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|(_, p, _)| p.name == "keep"));
}

// ─── Skill lookup ───────────────────────────────────────────────────────

#[test]
fn show_skill_detail() {
    let mut registry = Registry::default();
    let mut sk = skill("detailed-skill");
    sk.description = Some("Does important things".to_string());
    registry
        .sources
        .push(source("src", vec![plugin("myplugin", vec![sk])]));

    let (src_name, plugin_name, found) = registry.find_skill("myplugin/detailed-skill").unwrap();
    assert_eq!(src_name, "src");
    assert_eq!(plugin_name, "myplugin");
    assert_eq!(found.name, "detailed-skill");
    assert_eq!(found.description.as_deref(), Some("Does important things"));
}

#[test]
fn show_nonexistent_skill_error() {
    let registry = Registry::default();
    let result = registry.find_skill("nonexistent/skill");
    assert!(result.is_err());
}

#[test]
fn find_skill_full_form() {
    let mut registry = Registry::default();
    registry
        .sources
        .push(source("s", vec![plugin("p", vec![skill("sk")])]));

    let (src_name, plugin_name, found) = registry.find_skill("s:p/sk").unwrap();
    assert_eq!(src_name, "s");
    assert_eq!(plugin_name, "p");
    assert_eq!(found.name, "sk");
}
