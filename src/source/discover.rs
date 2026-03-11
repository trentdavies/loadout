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
