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
    /// Whether this plugin has an explicit .claude-plugin/plugin.json manifest.
    pub has_manifest: bool,
}

/// A discovered skill (SKILL.md) within a plugin.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    /// Validated skill name from frontmatter.
    pub name: String,
    /// Skill description from frontmatter.
    pub description: Option<String>,
    /// Optional author from metadata.author frontmatter.
    pub author: Option<String>,
    /// Optional version from metadata.version frontmatter.
    pub version: Option<String>,
    /// Path to the skill directory.
    pub path: PathBuf,
}

/// Discover plugins within a source directory.
///
/// Scans subdirectories for:
/// 1. Explicit plugins — subdirs containing .claude-plugin/plugin.json
/// 2. Implicit plugins — subdirs containing skill directories (SKILL.md)
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

        // Explicit: has .claude-plugin/plugin.json
        if dir_path.join(".claude-plugin/plugin.json").exists() {
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

        // Validate kebab-case
        if !detect::is_kebab_case(&skill_name) {
            eprintln!("warning: skipping {}: skill name '{}' is not kebab-case", dir_name, skill_name);
            continue;
        }

        // If name doesn't match directory, warn and use directory name with no description
        if skill_name != dir_name {
            eprintln!(
                "warning: {}: frontmatter name '{}' does not match directory name, using '{}'",
                dir_name, skill_name, dir_name
            );
            skills.push(DiscoveredSkill {
                name: dir_name,
                description: None,
                author: None,
                version: None,
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

        let author = detect::parse_skill_author(&skill_md);
        let version = detect::parse_skill_version(&skill_md);

        skills.push(DiscoveredSkill {
            name: skill_name,
            description,
            author,
            version,
            path: entry.path(),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill_dir(parent: &Path, name: &str, frontmatter: &str) {
        let dir = parent.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), frontmatter).unwrap();
    }

    // -- discover_plugins() tests --

    #[test]
    fn discover_plugins_multi_plugin() {
        let tmp = TempDir::new().unwrap();
        // Create two plugin dirs
        let p1 = tmp.path().join("alpha");
        let p2 = tmp.path().join("beta");
        fs::create_dir_all(&p1).unwrap();
        fs::create_dir_all(&p2).unwrap();
        // alpha has .claude-plugin/plugin.json
        let cp1 = p1.join(".claude-plugin");
        fs::create_dir_all(&cp1).unwrap();
        fs::write(cp1.join("plugin.json"), r#"{"name": "alpha"}"#).unwrap();
        // also needs skills to be valid
        make_skill_dir(&p1.join("skills"), "sk-a", "---\nname: sk-a\ndescription: d\n---\n");
        // beta has a skill subdir (implicit plugin)
        make_skill_dir(&p2, "my-skill", "---\nname: my-skill\ndescription: d\n---\n");

        let plugins = discover_plugins(tmp.path()).unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].dir_name, "alpha");
        assert!(plugins[0].has_manifest);
        assert_eq!(plugins[1].dir_name, "beta");
        assert!(!plugins[1].has_manifest);
    }

    #[test]
    fn discover_plugins_skips_hidden() {
        let tmp = TempDir::new().unwrap();
        let hidden = tmp.path().join(".hidden");
        fs::create_dir_all(&hidden).unwrap();
        let cp = hidden.join(".claude-plugin");
        fs::create_dir_all(&cp).unwrap();
        fs::write(cp.join("plugin.json"), r#"{"name": "hidden"}"#).unwrap();

        let plugins = discover_plugins(tmp.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn discover_plugins_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let plugins = discover_plugins(tmp.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn discover_plugins_sorted_alphabetically() {
        let tmp = TempDir::new().unwrap();
        for name in &["zeta", "alpha", "mid"] {
            let d = tmp.path().join(name);
            let cp = d.join(".claude-plugin");
            fs::create_dir_all(&cp).unwrap();
            fs::write(cp.join("plugin.json"), format!(r#"{{"name": "{}"}}"#, name)).unwrap();
            // Need skills for discovery
            make_skill_dir(&d.join("skills"), "sk", "---\nname: sk\ndescription: d\n---\n");
        }
        let plugins = discover_plugins(tmp.path()).unwrap();
        let names: Vec<&str> = plugins.iter().map(|p| p.dir_name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mid", "zeta"]);
    }

    // -- discover_skills() tests --

    #[test]
    fn discover_skills_happy_path() {
        let tmp = TempDir::new().unwrap();
        make_skill_dir(tmp.path(), "skill-a", "---\nname: skill-a\ndescription: A\n---\n");
        make_skill_dir(tmp.path(), "skill-b", "---\nname: skill-b\ndescription: B\n---\n");

        let skills = discover_skills(tmp.path()).unwrap();
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "skill-a");
        assert_eq!(skills[1].name, "skill-b");
    }

    #[test]
    fn discover_skills_missing_frontmatter_skipped() {
        let tmp = TempDir::new().unwrap();
        make_skill_dir(tmp.path(), "good", "---\nname: good\ndescription: ok\n---\n");
        make_skill_dir(tmp.path(), "bad", "no frontmatter here");

        let skills = discover_skills(tmp.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "good");
    }

    #[test]
    fn discover_skills_missing_description_skipped() {
        let tmp = TempDir::new().unwrap();
        make_skill_dir(tmp.path(), "no-desc", "---\nname: no-desc\n---\nbody");

        let skills = discover_skills(tmp.path()).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn discover_skills_empty_plugin() {
        let tmp = TempDir::new().unwrap();
        let skills = discover_skills(tmp.path()).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn discover_skills_sorted() {
        let tmp = TempDir::new().unwrap();
        make_skill_dir(tmp.path(), "zebra", "---\nname: zebra\ndescription: z\n---\n");
        make_skill_dir(tmp.path(), "apple", "---\nname: apple\ndescription: a\n---\n");

        let skills = discover_skills(tmp.path()).unwrap();
        assert_eq!(skills[0].name, "apple");
        assert_eq!(skills[1].name, "zebra");
    }

    #[test]
    fn discover_skills_uses_skills_subdir() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        make_skill_dir(&skills_dir, "inner", "---\nname: inner\ndescription: d\n---\n");

        let skills = discover_skills(tmp.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "inner");
    }
}
