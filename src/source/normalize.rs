use std::path::Path;
use anyhow::Result;

use super::detect::{self, SourceStructure};
use super::{discover, manifest};
use crate::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource};

/// Normalize a detected source into the canonical Source > Plugin > Skill hierarchy.
pub fn normalize(
    source_name: &str,
    cache_path: &Path,
    structure: &SourceStructure,
) -> Result<RegisteredSource> {
    let plugins = match structure {
        SourceStructure::SingleFile { skill_name } => {
            let skill = scan_single_file_skill(skill_name, cache_path)?;
            vec![RegisteredPlugin {
                name: source_name.to_string(),
                version: None,
                description: None,
                skills: vec![skill],
                path: cache_path.to_path_buf(),
            }]
        }

        SourceStructure::Marketplace => {
            scan_marketplace(cache_path)?
        }

        SourceStructure::SinglePlugin => {
            let plugin = scan_plugin_dir(cache_path)?;
            vec![plugin]
        }

        SourceStructure::FlatSkills => {
            let dir_name = cache_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(source_name);
            let discovered = discover::discover_skills(cache_path)?;
            let skills = discovered.into_iter().map(|ds| RegisteredSkill {
                name: ds.name,
                description: ds.description,
                author: ds.author,
                version: ds.version,
                path: ds.path,
            }).collect();
            vec![RegisteredPlugin {
                name: dir_name.to_string(),
                version: None,
                description: None,
                skills,
                path: cache_path.to_path_buf(),
            }]
        }

        SourceStructure::SingleSkillDir { skill_name } => {
            let skill = scan_skill_dir(skill_name, cache_path)?;
            vec![RegisteredPlugin {
                name: source_name.to_string(),
                version: None,
                description: None,
                skills: vec![skill],
                path: cache_path.to_path_buf(),
            }]
        }
    };

    Ok(RegisteredSource {
        name: source_name.to_string(),
        plugins,
        cache_path: cache_path.to_path_buf(),
    })
}

/// Scan a marketplace (has .claude-plugin/marketplace.json).
fn scan_marketplace(path: &Path) -> Result<Vec<RegisteredPlugin>> {
    let manifest_path = path.join(".claude-plugin/marketplace.json");
    let marketplace = manifest::load_marketplace(&manifest_path)?;

    let mut plugins = Vec::new();
    for mp in &marketplace.plugins {
        // Resolve plugin path relative to source root
        let plugin_path = path.join(mp.source.trim_start_matches("./"));
        if !plugin_path.is_dir() {
            eprintln!("warning: marketplace plugin '{}' path does not exist: {}", mp.name, mp.source);
            continue;
        }

        match scan_plugin_dir(&plugin_path) {
            Ok(mut plugin) => {
                // Marketplace description supplements plugin.json description
                if plugin.description.is_none() {
                    plugin.description = mp.description.clone();
                }
                plugins.push(plugin);
            }
            Err(e) => {
                eprintln!("warning: skipping marketplace plugin '{}': {}", mp.name, e);
            }
        }
    }

    Ok(plugins)
}

/// Scan a plugin directory. Reads .claude-plugin/plugin.json for metadata.
fn scan_plugin_dir(path: &Path) -> Result<RegisteredPlugin> {
    let plugin_json = path.join(".claude-plugin/plugin.json");

    let (name, version, description) = if plugin_json.exists() {
        let m = manifest::load_plugin_manifest(&plugin_json)?;
        (
            m.name,
            m.version,
            m.description,
        )
    } else {
        let n = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        (n, None, None)
    };

    let discovered = discover::discover_skills(path)?;
    let skills = discovered.into_iter().map(|ds| RegisteredSkill {
        name: ds.name,
        description: ds.description,
        author: ds.author,
        version: ds.version,
        path: ds.path,
    }).collect();

    Ok(RegisteredPlugin {
        name,
        version,
        description,
        skills,
        path: path.to_path_buf(),
    })
}

/// Create a RegisteredSkill from a single SKILL.md file in the cache root.
fn scan_single_file_skill(skill_name: &str, cache_path: &Path) -> Result<RegisteredSkill> {
    let skill_file = cache_path.join(format!("{}.md", skill_name));
    let skill_file = if skill_file.exists() { skill_file } else { cache_path.join("SKILL.md") };

    if skill_file.exists() && !detect::has_skill_frontmatter(&skill_file) {
        eprintln!("warning: skill file has no valid frontmatter (name and description required): {}", skill_file.display());
        anyhow::bail!("skill file missing required frontmatter (name and description)");
    }

    let name = detect::parse_skill_name(&skill_file).unwrap_or_else(|| skill_name.to_string());
    let description = detect::parse_skill_description(&skill_file);
    let author = detect::parse_skill_author(&skill_file);
    let version = detect::parse_skill_version(&skill_file);

    Ok(RegisteredSkill {
        name,
        description,
        author,
        version,
        path: cache_path.to_path_buf(),
    })
}

/// Create a RegisteredSkill from a directory containing SKILL.md.
fn scan_skill_dir(skill_name: &str, path: &Path) -> Result<RegisteredSkill> {
    let skill_md = path.join("SKILL.md");

    if skill_md.exists() && !detect::has_skill_frontmatter(&skill_md) {
        eprintln!("warning: SKILL.md has no valid frontmatter (name and description required): {}", skill_md.display());
        anyhow::bail!("SKILL.md missing required frontmatter (name and description)");
    }

    let name = if skill_md.exists() {
        detect::parse_skill_name(&skill_md).unwrap_or_else(|| skill_name.to_string())
    } else {
        skill_name.to_string()
    };
    let description = if skill_md.exists() { detect::parse_skill_description(&skill_md) } else { None };
    let author = if skill_md.exists() { detect::parse_skill_author(&skill_md) } else { None };
    let version = if skill_md.exists() { detect::parse_skill_version(&skill_md) } else { None };

    Ok(RegisteredSkill {
        name,
        description,
        author,
        version,
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill_md(dir: &Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {}\ndescription: A skill\n---\nbody", name),
        ).unwrap();
    }

    fn make_plugin_json(dir: &Path, json: &str) {
        let cp = dir.join(".claude-plugin");
        fs::create_dir_all(&cp).unwrap();
        fs::write(cp.join("plugin.json"), json).unwrap();
    }

    fn make_marketplace(dir: &Path, json: &str) {
        let cp = dir.join(".claude-plugin");
        fs::create_dir_all(&cp).unwrap();
        fs::write(cp.join("marketplace.json"), json).unwrap();
    }

    #[test]
    fn normalize_single_file() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("test.md"), "---\nname: test\ndescription: desc\n---\n").unwrap();

        let structure = SourceStructure::SingleFile { skill_name: "test".to_string() };
        let result = normalize("my-src", &cache, &structure).unwrap();
        assert_eq!(result.name, "my-src");
        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].skills.len(), 1);
    }

    #[test]
    fn normalize_flat_skills() {
        let tmp = TempDir::new().unwrap();
        make_skill_md(&tmp.path().join("skill-a"), "skill-a");
        make_skill_md(&tmp.path().join("skill-b"), "skill-b");

        let structure = SourceStructure::FlatSkills;
        let result = normalize("src", tmp.path(), &structure).unwrap();
        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].skills.len(), 2);
    }

    #[test]
    fn normalize_single_plugin_with_plugin_json() {
        let tmp = TempDir::new().unwrap();
        make_plugin_json(tmp.path(), r#"{"name": "my-plug", "version": "1.0", "description": "A plugin"}"#);
        make_skill_md(&tmp.path().join("skills").join("skill-x"), "skill-x");

        let structure = SourceStructure::SinglePlugin;
        let result = normalize("src", tmp.path(), &structure).unwrap();
        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].name, "my-plug");
        assert_eq!(result.plugins[0].version.as_deref(), Some("1.0"));
        assert_eq!(result.plugins[0].skills.len(), 1);
    }

    #[test]
    fn normalize_single_plugin_no_manifest() {
        let tmp = TempDir::new().unwrap();
        make_skill_md(&tmp.path().join("skills").join("skill-x"), "skill-x");

        let structure = SourceStructure::SinglePlugin;
        let result = normalize("src", tmp.path(), &structure).unwrap();
        assert_eq!(result.plugins.len(), 1);
        // Name falls back to directory name
        assert_eq!(result.plugins[0].skills.len(), 1);
    }

    #[test]
    fn normalize_marketplace() {
        let tmp = TempDir::new().unwrap();

        // Create marketplace
        make_marketplace(tmp.path(), r#"{
            "name": "test-marketplace",
            "plugins": [
                {"name": "legal", "source": "./legal", "description": "Legal tools"},
                {"name": "sales", "source": "./sales"}
            ]
        }"#);

        // Create plugin dirs with .claude-plugin/plugin.json and skills
        let legal = tmp.path().join("legal");
        make_plugin_json(&legal, r#"{"name": "legal", "version": "1.1.0", "description": "Legal"}"#);
        make_skill_md(&legal.join("skills").join("contract-review"), "contract-review");
        make_skill_md(&legal.join("skills").join("nda-triage"), "nda-triage");

        let sales = tmp.path().join("sales");
        make_plugin_json(&sales, r#"{"name": "sales", "version": "2.0"}"#);
        make_skill_md(&sales.join("skills").join("call-prep"), "call-prep");

        let structure = SourceStructure::Marketplace;
        let result = normalize("mkt", tmp.path(), &structure).unwrap();
        assert_eq!(result.plugins.len(), 2);
        assert_eq!(result.plugins[0].name, "legal");
        assert_eq!(result.plugins[0].version.as_deref(), Some("1.1.0"));
        assert_eq!(result.plugins[0].skills.len(), 2);
        assert_eq!(result.plugins[1].name, "sales");
        assert_eq!(result.plugins[1].skills.len(), 1);
    }

    #[test]
    fn normalize_marketplace_missing_plugin_dir_skipped() {
        let tmp = TempDir::new().unwrap();
        make_marketplace(tmp.path(), r#"{
            "name": "mkt",
            "plugins": [
                {"name": "exists", "source": "./exists"},
                {"name": "missing", "source": "./missing"}
            ]
        }"#);

        let exists = tmp.path().join("exists");
        make_skill_md(&exists.join("skills").join("sk"), "sk");

        let structure = SourceStructure::Marketplace;
        let result = normalize("mkt", tmp.path(), &structure).unwrap();
        // Only 'exists' should be included
        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].name, "exists");
    }

    #[test]
    fn normalize_single_skill_dir() {
        let tmp = TempDir::new().unwrap();
        make_skill_md(tmp.path(), "dir-skill");

        let structure = SourceStructure::SingleSkillDir { skill_name: "dir-skill".to_string() };
        let result = normalize("src", tmp.path(), &structure).unwrap();
        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].skills[0].name, "dir-skill");
    }
}
