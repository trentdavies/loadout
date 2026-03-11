use std::fs;
use std::path::Path;
use anyhow::{bail, Result};

/// Detected source structure.
#[derive(Debug, Clone)]
pub enum SourceStructure {
    /// A single SKILL.md file (path points to the file).
    SingleFile { skill_name: String },
    /// Directory with source.toml — multi-plugin source.
    FullSource,
    /// Directory with plugin.toml — single plugin.
    SinglePlugin,
    /// Directory with subdirs containing SKILL.md — flat plugin (inferred).
    FlatSkills,
    /// Directory containing SKILL.md directly — single skill dir.
    SingleSkillDir { skill_name: String },
}

/// Detect the structure of a source at the given path.
///
/// Detection order:
/// 1. Single file with YAML frontmatter → SingleFile
/// 2. Directory with source.toml → FullSource
/// 3. Directory with plugin.toml → SinglePlugin
/// 4. Directory with subdirs containing SKILL.md → FlatSkills
/// 5. Directory containing SKILL.md directly → SingleSkillDir
/// 6. Error
pub fn detect(path: &Path) -> Result<SourceStructure> {
    // 1. Single file
    if path.is_file() {
        let name = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        if has_skill_frontmatter(path) {
            return Ok(SourceStructure::SingleFile { skill_name: name });
        }
        bail!(
            "file does not appear to be a valid skill (no YAML frontmatter with name/description): {}",
            path.display()
        );
    }

    if !path.is_dir() {
        bail!("source path is not a file or directory: {}", path.display());
    }

    // 2. source.toml
    if path.join("source.toml").exists() {
        return Ok(SourceStructure::FullSource);
    }

    // 3. plugin.toml
    if path.join("plugin.toml").exists() {
        return Ok(SourceStructure::SinglePlugin);
    }

    // 4. Subdirs with SKILL.md
    if has_skill_subdirs(path) {
        return Ok(SourceStructure::FlatSkills);
    }

    // 5. SKILL.md in this directory
    if path.join("SKILL.md").exists() {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        return Ok(SourceStructure::SingleSkillDir { skill_name: name });
    }

    // 6. Error
    bail!(
        "cannot determine source structure at: {}\n\
         Expected one of:\n\
         - A SKILL.md file with YAML frontmatter\n\
         - A directory with source.toml (multi-plugin source)\n\
         - A directory with plugin.toml (single plugin)\n\
         - A directory with subdirectories containing SKILL.md files\n\
         - A directory containing SKILL.md directly",
        path.display()
    );
}

/// Check if a file has YAML frontmatter with `name:` and `description:` fields.
pub fn has_skill_frontmatter(path: &Path) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Check for YAML frontmatter delimiters
    if !content.starts_with("---") {
        return false;
    }

    // Find the closing ---
    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(pos) => pos,
        None => return false,
    };

    let frontmatter = &rest[..end];
    frontmatter.contains("name:") && frontmatter.contains("description:")
}

/// Check if any subdirectories contain SKILL.md files.
pub fn has_skill_subdirs(path: &Path) -> bool {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if entry.path().join("SKILL.md").exists() {
                return true;
            }
        }
    }

    false
}

/// Parse the `name` field from SKILL.md YAML frontmatter.
pub fn parse_skill_name(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("name:") {
            return Some(value.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

/// Parse the `description` field from SKILL.md YAML frontmatter.
pub fn parse_skill_description(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("description:") {
            return Some(value.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_skill(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    // -- detect() tests --

    #[test]
    fn detect_single_file() {
        let tmp = TempDir::new().unwrap();
        let skill = write_skill(tmp.path(), "SKILL.md", "---\nname: test\ndescription: A test\n---\nbody");
        match detect(&skill).unwrap() {
            SourceStructure::SingleFile { skill_name } => assert_eq!(skill_name, "SKILL"),
            other => panic!("expected SingleFile, got {:?}", other),
        }
    }

    #[test]
    fn detect_full_source() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("source.toml"), "name = \"src\"").unwrap();
        match detect(tmp.path()).unwrap() {
            SourceStructure::FullSource => {}
            other => panic!("expected FullSource, got {:?}", other),
        }
    }

    #[test]
    fn detect_single_plugin() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("plugin.toml"), "name = \"plug\"").unwrap();
        match detect(tmp.path()).unwrap() {
            SourceStructure::SinglePlugin => {}
            other => panic!("expected SinglePlugin, got {:?}", other),
        }
    }

    #[test]
    fn detect_flat_skills() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("my-skill");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "---\nname: my-skill\ndescription: d\n---\n").unwrap();
        match detect(tmp.path()).unwrap() {
            SourceStructure::FlatSkills => {}
            other => panic!("expected FlatSkills, got {:?}", other),
        }
    }

    #[test]
    fn detect_single_skill_dir() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "---\nname: x\ndescription: d\n---\n").unwrap();
        match detect(tmp.path()).unwrap() {
            SourceStructure::SingleSkillDir { skill_name } => {
                assert!(!skill_name.is_empty());
            }
            other => panic!("expected SingleSkillDir, got {:?}", other),
        }
    }

    #[test]
    fn detect_empty_dir_errors() {
        let tmp = TempDir::new().unwrap();
        assert!(detect(tmp.path()).is_err());
    }

    #[test]
    fn detect_nonexistent_path_errors() {
        assert!(detect(Path::new("/nonexistent/path/xyz")).is_err());
    }

    #[test]
    fn detect_file_without_frontmatter_errors() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "no frontmatter here");
        assert!(detect(&f).is_err());
    }

    // -- has_skill_frontmatter() tests --

    #[test]
    fn has_frontmatter_valid() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\ndescription: d\n---\nbody");
        assert!(has_skill_frontmatter(&f));
    }

    #[test]
    fn has_frontmatter_missing() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "just text, no frontmatter");
        assert!(!has_skill_frontmatter(&f));
    }

    #[test]
    fn has_frontmatter_name_only() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\n---\nbody");
        // requires both name: and description:
        assert!(!has_skill_frontmatter(&f));
    }

    #[test]
    fn has_frontmatter_no_closing_delimiter() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\ndescription: d\nno closing");
        assert!(!has_skill_frontmatter(&f));
    }

    #[test]
    fn has_frontmatter_nonexistent_file() {
        assert!(!has_skill_frontmatter(Path::new("/nonexistent/SKILL.md")));
    }

    // -- parse_skill_name() tests --

    #[test]
    fn parse_name_present() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: my-skill\ndescription: d\n---\n");
        assert_eq!(parse_skill_name(&f), Some("my-skill".to_string()));
    }

    #[test]
    fn parse_name_missing() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\ndescription: d\n---\n");
        assert_eq!(parse_skill_name(&f), None);
    }

    #[test]
    fn parse_name_no_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "no frontmatter");
        assert_eq!(parse_skill_name(&f), None);
    }

    #[test]
    fn parse_name_quoted() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: \"quoted-name\"\ndescription: d\n---\n");
        assert_eq!(parse_skill_name(&f), Some("quoted-name".to_string()));
    }

    #[test]
    fn parse_name_with_whitespace() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname:   spaced  \ndescription: d\n---\n");
        assert_eq!(parse_skill_name(&f), Some("spaced".to_string()));
    }

    // -- parse_skill_description() tests --

    #[test]
    fn parse_description_present() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\ndescription: A test skill\n---\n");
        assert_eq!(parse_skill_description(&f), Some("A test skill".to_string()));
    }

    #[test]
    fn parse_description_missing() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\n---\n");
        assert_eq!(parse_skill_description(&f), None);
    }

    #[test]
    fn parse_description_quoted() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\nname: x\ndescription: \"quoted desc\"\n---\n");
        assert_eq!(parse_skill_description(&f), Some("quoted desc".to_string()));
    }

    // -- has_skill_subdirs() tests --

    #[test]
    fn has_skill_subdirs_true() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("skill-a");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "content").unwrap();
        assert!(has_skill_subdirs(tmp.path()));
    }

    #[test]
    fn has_skill_subdirs_false_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_skill_subdirs(tmp.path()));
    }

    #[test]
    fn has_skill_subdirs_false_no_skill_md() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("not-a-skill");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("README.md"), "content").unwrap();
        assert!(!has_skill_subdirs(tmp.path()));
    }

    // -- frontmatter edge cases --

    #[test]
    fn frontmatter_empty_delimiters() {
        let tmp = TempDir::new().unwrap();
        let f = write_skill(tmp.path(), "SKILL.md", "---\n---\nbody");
        assert!(!has_skill_frontmatter(&f)); // no name: or description:
        assert_eq!(parse_skill_name(&f), None);
    }
}
