use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

use super::detect::{self, SourceStructure};
use crate::registry::{RegisteredPlugin, RegisteredSkill, RegisteredSource};

/// Source manifest (source.toml).
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct SourceManifest {
    name: String,
    version: Option<String>,
    description: Option<String>,
    plugins: Option<Vec<String>>,
}

/// Plugin manifest (plugin.toml).
#[derive(Debug, serde::Deserialize)]
struct PluginManifest {
    name: String,
    version: Option<String>,
    description: Option<String>,
}

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
            let skills = scan_skill_subdirs(cache_path)?;
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
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: SourceManifest = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let plugin_dirs: Vec<String> = if let Some(explicit) = manifest.plugins {
        explicit
    } else {
        // Auto-discover: scan subdirs for plugin.toml
        let mut dirs = Vec::new();
        for entry in fs::read_dir(path)?.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if entry.path().join("plugin.toml").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        dirs.push(name.to_string());
                    }
                }
            }
        }
        dirs.sort();
        dirs
    };

    let mut plugins = Vec::new();
    for dir_name in &plugin_dirs {
        let plugin_path = path.join(dir_name);
        if plugin_path.is_dir() {
            match scan_plugin_dir(&plugin_path) {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => {
                    eprintln!("warning: skipping plugin {}: {}", dir_name, e);
                }
            }
        }
    }

    Ok(plugins)
}

/// Scan a directory with plugin.toml.
fn scan_plugin_dir(path: &Path) -> Result<RegisteredPlugin> {
    let manifest_path = path.join("plugin.toml");
    let (name, version, description) = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
        (manifest.name, manifest.version, manifest.description)
    } else {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        (name, None, None)
    };

    // Look for skills in skills/ subdir or directly in the plugin dir
    let skills_dir = path.join("skills");
    let skills = if skills_dir.is_dir() {
        scan_skill_subdirs(&skills_dir)?
    } else {
        scan_skill_subdirs(path)?
    };

    Ok(RegisteredPlugin {
        name,
        version,
        description,
        skills,
        path: path.to_path_buf(),
    })
}

/// Scan subdirectories for SKILL.md files.
fn scan_skill_subdirs(path: &Path) -> Result<Vec<RegisteredSkill>> {
    let mut skills = Vec::new();

    let entries = fs::read_dir(path)
        .with_context(|| format!("failed to read dir: {}", path.display()))?;

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Validate frontmatter
        let parsed_name = detect::parse_skill_name(&skill_md);
        let description = detect::parse_skill_description(&skill_md);

        if parsed_name.is_none() {
            eprintln!("warning: skipping {}: no valid frontmatter in SKILL.md", dir_name);
            continue;
        }

        let skill_name = parsed_name.unwrap();
        if skill_name != dir_name {
            eprintln!(
                "warning: skipping {}: frontmatter name '{}' does not match directory name",
                dir_name, skill_name
            );
            continue;
        }

        skills.push(RegisteredSkill {
            name: skill_name,
            description,
            path: entry.path(),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Create a RegisteredSkill from a single SKILL.md file in the cache root.
fn scan_single_file_skill(skill_name: &str, cache_path: &Path) -> Result<RegisteredSkill> {
    let description = detect::parse_skill_description(
        &cache_path.join(format!("{}.md", skill_name))
    ).or_else(|| {
        // Try SKILL.md directly
        detect::parse_skill_description(&cache_path.join("SKILL.md"))
    });

    Ok(RegisteredSkill {
        name: skill_name.to_string(),
        description,
        path: cache_path.to_path_buf(),
    })
}

/// Create a RegisteredSkill from a directory containing SKILL.md.
fn scan_skill_dir(skill_name: &str, path: &Path) -> Result<RegisteredSkill> {
    let skill_md = path.join("SKILL.md");
    let description = if skill_md.exists() {
        detect::parse_skill_description(&skill_md)
    } else {
        None
    };

    Ok(RegisteredSkill {
        name: skill_name.to_string(),
        description,
        path: path.to_path_buf(),
    })
}
