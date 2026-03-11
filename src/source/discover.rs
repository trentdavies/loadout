use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

use super::detect;

/// A discovered plugin directory within a source.
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Directory name / plugin name.
    pub dir_name: String,
    /// Absolute path to the plugin directory.
    pub path: PathBuf,
    /// Whether this plugin has an explicit plugin.toml manifest.
    pub has_manifest: bool,
}

/// A discovered skill (SKILL.md) within a plugin.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    /// Validated skill name from frontmatter.
    pub name: String,
    /// Skill description from frontmatter.
    pub description: Option<String>,
    /// Path to the skill directory.
    pub path: PathBuf,
}

/// Discover plugins within a source directory.
///
/// Scans subdirectories for:
/// 1. Explicit plugins — subdirs containing plugin.toml
/// 2. Implicit plugins — subdirs containing skill directories (SKILL.md) but no plugin.toml
///
/// Skips hidden directories (starting with '.').
/// Returns results sorted by directory name.
pub fn discover_plugins(source_path: &Path) -> Result<Vec<DiscoveredPlugin>> {
    let mut plugins = Vec::new();

    let entries = fs::read_dir(source_path)?;

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }

        let dir_name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip hidden directories
        if dir_name.starts_with('.') {
            continue;
        }

        let dir_path = entry.path();

        // Explicit: has plugin.toml
        if dir_path.join("plugin.toml").exists() {
            plugins.push(DiscoveredPlugin {
                dir_name,
                path: dir_path,
                has_manifest: true,
            });
            continue;
        }

        // Implicit: has skill subdirs (or a skills/ subdir with skill subdirs)
        if has_skills(&dir_path) {
            plugins.push(DiscoveredPlugin {
                dir_name,
                path: dir_path,
                has_manifest: false,
            });
        }
    }

    plugins.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    Ok(plugins)
}

/// Check if a directory contains skills — either directly or in a skills/ subdir.
fn has_skills(path: &Path) -> bool {
    // Check skills/ subdirectory first
    let skills_dir = path.join("skills");
    if skills_dir.is_dir() && detect::has_skill_subdirs(&skills_dir) {
        return true;
    }

    // Check direct subdirs for SKILL.md
    detect::has_skill_subdirs(path)
}

/// Discover skills within a plugin directory.
///
/// Scans subdirectories for SKILL.md files and validates frontmatter:
/// - `name` is required
/// - `description` is required
/// - `name` must match the directory name
///
/// Checks both direct subdirs and a `skills/` subdirectory.
/// Returns validated skills sorted by name. Invalid skills are skipped with warnings.
pub fn discover_skills(plugin_path: &Path) -> Result<Vec<DiscoveredSkill>> {
    let skills_dir = plugin_path.join("skills");
    let scan_path = if skills_dir.is_dir() {
        &skills_dir
    } else {
        plugin_path
    };

    scan_skill_dirs(scan_path)
}

/// Scan a directory's subdirectories for valid SKILL.md files.
fn scan_skill_dirs(path: &Path) -> Result<Vec<DiscoveredSkill>> {
    let mut skills = Vec::new();

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(skills),
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }

        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Validate frontmatter: name required
        let parsed_name = detect::parse_skill_name(&skill_md);
        if parsed_name.is_none() {
            eprintln!("warning: skipping {}: SKILL.md missing required 'name' in frontmatter", dir_name);
            continue;
        }

        let skill_name = parsed_name.unwrap();

        // If name doesn't match directory, warn and use directory name with no description
        // (to avoid leaking potentially wrong frontmatter data)
        if skill_name != dir_name {
            eprintln!(
                "warning: {}: frontmatter name '{}' does not match directory name, using '{}'",
                dir_name, skill_name, dir_name
            );
            skills.push(DiscoveredSkill {
                name: dir_name,
                description: None,
                path: entry.path(),
            });
            continue;
        }

        // Validate frontmatter: description required
        let description = detect::parse_skill_description(&skill_md);
        if description.is_none() {
            eprintln!("warning: skipping {}: SKILL.md missing required 'description' in frontmatter", dir_name);
            continue;
        }

        skills.push(DiscoveredSkill {
            name: skill_name,
            description,
            path: entry.path(),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}
