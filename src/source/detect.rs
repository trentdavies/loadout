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
fn has_skill_frontmatter(path: &Path) -> bool {
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
fn has_skill_subdirs(path: &Path) -> bool {
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
