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

        SourceStructure::FullSource => {
            scan_full_source(cache_path)?
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

/// Scan a full source (has source.toml).
fn scan_full_source(path: &Path) -> Result<Vec<RegisteredPlugin>> {
    let manifest_path = path.join("source.toml");
    let source_manifest = manifest::load_source_manifest(&manifest_path)?;

    // If source.toml lists explicit plugins, use those directories.
    // Otherwise, discover plugins automatically (explicit + implicit).
    let discovered = if let Some(explicit) = source_manifest.plugins {
        explicit.into_iter()
            .filter_map(|name| {
                let p = path.join(&name);
                if p.is_dir() {
                    Some(discover::DiscoveredPlugin {
                        dir_name: name,
                        path: p.clone(),
                        has_manifest: p.join("plugin.toml").exists(),
                    })
                } else {
                    eprintln!("warning: listed plugin dir '{}' does not exist", name);
                    None
                }
            })
            .collect()
    } else {
        discover::discover_plugins(path)?
    };

    let mut plugins = Vec::new();
    for dp in &discovered {
        match scan_plugin_dir(&dp.path) {
            Ok(plugin) => plugins.push(plugin),
            Err(e) => {
                eprintln!("warning: skipping plugin {}: {}", dp.dir_name, e);
            }
        }
    }

    Ok(plugins)
}

/// Scan a directory with plugin.toml.
fn scan_plugin_dir(path: &Path) -> Result<RegisteredPlugin> {
    let manifest_path = path.join("plugin.toml");
    let (name, version, description) = if manifest_path.exists() {
        let m = manifest::load_plugin_manifest(&manifest_path)?;
        (m.name, m.version, m.description)
    } else {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        (name, None, None)
    };

    // Discover skills using the shared discovery function
    let discovered = discover::discover_skills(path)?;
    let skills = discovered.into_iter().map(|ds| RegisteredSkill {
        name: ds.name,
        description: ds.description,
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
    // Try the renamed file first, then SKILL.md
    let skill_file = cache_path.join(format!("{}.md", skill_name));
    let skill_file = if skill_file.exists() { skill_file } else { cache_path.join("SKILL.md") };

    if skill_file.exists() && !detect::has_skill_frontmatter(&skill_file) {
        eprintln!("warning: skill file has no valid frontmatter (name and description required): {}", skill_file.display());
        anyhow::bail!("skill file missing required frontmatter (name and description)");
    }

    // Use frontmatter name if available, fall back to provided name
    let name = detect::parse_skill_name(&skill_file).unwrap_or_else(|| skill_name.to_string());
    let description = detect::parse_skill_description(&skill_file);

    Ok(RegisteredSkill {
        name,
        description,
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

    // Use frontmatter name if available, fall back to provided name
    let name = if skill_md.exists() {
        detect::parse_skill_name(&skill_md).unwrap_or_else(|| skill_name.to_string())
    } else {
        skill_name.to_string()
    };
    let description = if skill_md.exists() {
        detect::parse_skill_description(&skill_md)
    } else {
        None
    };

    Ok(RegisteredSkill {
        name,
        description,
        path: path.to_path_buf(),
    })
}
