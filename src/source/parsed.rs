use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use super::{detect, manifest};

/// The structural kind of a parsed source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    SingleFile,
    Marketplace,
    SinglePlugin,
    FlatSkills,
    SingleSkillDir,
}

/// Parsed source metadata shared across detection, prompting, and normalization.
#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub kind: SourceKind,
    pub source_name: String,
    pub display_name: Option<String>,
    pub url: Option<String>,
    pub path: PathBuf,
    pub plugin_name: Option<String>,
    pub skill_name: Option<String>,
}

impl ParsedSource {
    /// Parse a source path into a shared representation with inferred names.
    pub fn parse(path: &Path) -> Result<Self> {
        let source_name = default_source_name(path);

        // 1. Single file
        if path.is_file() {
            let skill_name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string();

            if detect::has_skill_frontmatter(path) {
                return Ok(Self {
                    kind: SourceKind::SingleFile,
                    source_name,
                    display_name: None,
                    url: None,
                    path: path.to_path_buf(),
                    plugin_name: None,
                    skill_name: Some(skill_name),
                });
            }

            bail!(
                "file does not appear to be a valid skill (no YAML frontmatter with name/description): {}",
                path.display()
            );
        }

        if !path.is_dir() {
            bail!("source path is not a file or directory: {}", path.display());
        }

        // 2. Marketplace
        if path.join(".claude-plugin/marketplace.json").exists() {
            let display_name = manifest::load_marketplace(&path.join(".claude-plugin/marketplace.json"))?
                .name;
            return Ok(Self {
                kind: SourceKind::Marketplace,
                source_name,
                display_name: Some(display_name),
                url: None,
                path: path.to_path_buf(),
                plugin_name: None,
                skill_name: None,
            });
        }

        // 3. Single plugin
        let plugin_json = path.join(".claude-plugin/plugin.json");
        if plugin_json.exists() {
            let plugin_name = manifest::load_plugin_manifest(&plugin_json)?.name;
            return Ok(Self {
                kind: SourceKind::SinglePlugin,
                source_name,
                display_name: None,
                url: None,
                path: path.to_path_buf(),
                plugin_name: Some(plugin_name),
                skill_name: None,
            });
        }

        // 4. Flat skills
        if detect::has_skill_subdirs(path) {
            let plugin_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.strip_prefix('.').unwrap_or(n).to_string())
                .unwrap_or_else(|| source_name.clone());

            return Ok(Self {
                kind: SourceKind::FlatSkills,
                source_name,
                display_name: None,
                url: None,
                path: path.to_path_buf(),
                plugin_name: Some(plugin_name),
                skill_name: None,
            });
        }

        // 5. Single skill dir
        if path.join("SKILL.md").exists() {
            let skill_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string();

            return Ok(Self {
                kind: SourceKind::SingleSkillDir,
                source_name,
                display_name: None,
                url: None,
                path: path.to_path_buf(),
                plugin_name: None,
                skill_name: Some(skill_name),
            });
        }

        bail!(
            "cannot determine source structure at: {}\n\
             Expected one of:\n\
             - A SKILL.md file with YAML frontmatter\n\
             - A directory with .claude-plugin/marketplace.json (multi-plugin marketplace)\n\
             - A directory with .claude-plugin/plugin.json (single plugin)\n\
             - A directory with subdirectories containing SKILL.md files\n\
             - A directory containing SKILL.md directly",
            path.display()
        );
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = source_name.into();
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn default_plugin_name(&self) -> Option<&str> {
        match self.kind {
            SourceKind::SingleFile | SourceKind::SingleSkillDir => Some(&self.source_name),
            SourceKind::Marketplace => None,
            SourceKind::SinglePlugin | SourceKind::FlatSkills => self.plugin_name.as_deref(),
        }
    }

    pub fn prompt_plugin_name(&self) -> Option<&str> {
        let plugin_name = self.default_plugin_name()?;
        if plugin_name == self.source_name {
            None
        } else {
            Some(plugin_name)
        }
    }

    pub fn prompt_skill_name(&self) -> Option<&str> {
        match self.kind {
            SourceKind::SingleFile | SourceKind::SingleSkillDir => self.skill_name.as_deref(),
            SourceKind::Marketplace | SourceKind::SinglePlugin | SourceKind::FlatSkills => None,
        }
    }
}

fn default_source_name(path: &Path) -> String {
    if path.is_file() {
        path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string()
    } else {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string()
    }
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
            format!("---\nname: {}\ndescription: desc\n---\nbody", name),
        )
        .unwrap();
    }

    fn make_plugin_json(dir: &Path, json: &str) {
        let cp = dir.join(".claude-plugin");
        fs::create_dir_all(&cp).unwrap();
        fs::write(cp.join("plugin.json"), json).unwrap();
    }

    fn make_marketplace_json(dir: &Path) {
        let cp = dir.join(".claude-plugin");
        fs::create_dir_all(&cp).unwrap();
        fs::write(
            cp.join("marketplace.json"),
            r#"{"name":"mkt","plugins":[]}"#,
        )
        .unwrap();
    }

    #[test]
    fn parse_single_file_tracks_skill_name() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("SKILL.md");
        fs::write(&skill, "---\nname: skill\ndescription: desc\n---\n").unwrap();

        let parsed = ParsedSource::parse(&skill).unwrap();
        assert_eq!(parsed.kind, SourceKind::SingleFile);
        assert_eq!(parsed.skill_name.as_deref(), Some("SKILL"));
        assert_eq!(parsed.default_plugin_name(), Some("SKILL"));
    }

    #[test]
    fn parse_plugin_tracks_manifest_name() {
        let tmp = TempDir::new().unwrap();
        make_plugin_json(tmp.path(), r#"{"name":"manifest-plugin"}"#);
        make_skill_md(&tmp.path().join("skills").join("skill-a"), "skill-a");

        let parsed = ParsedSource::parse(tmp.path()).unwrap();
        assert_eq!(parsed.kind, SourceKind::SinglePlugin);
        assert_eq!(parsed.plugin_name.as_deref(), Some("manifest-plugin"));
        assert_eq!(parsed.prompt_plugin_name(), Some("manifest-plugin"));
    }

    #[test]
    fn parse_flat_skills_strips_hidden_prefix() {
        let tmp = TempDir::new().unwrap();
        let hidden = tmp.path().join(".curated");
        make_skill_md(&hidden.join("skill-a"), "skill-a");

        let parsed = ParsedSource::parse(&hidden).unwrap();
        assert_eq!(parsed.kind, SourceKind::FlatSkills);
        assert_eq!(parsed.plugin_name.as_deref(), Some("curated"));
    }

    #[test]
    fn parse_marketplace_has_no_prompt_names() {
        let tmp = TempDir::new().unwrap();
        make_marketplace_json(tmp.path());

        let parsed = ParsedSource::parse(tmp.path()).unwrap();
        assert_eq!(parsed.kind, SourceKind::Marketplace);
        assert_eq!(parsed.display_name.as_deref(), Some("mkt"));
        assert!(parsed.prompt_plugin_name().is_none());
        assert!(parsed.prompt_skill_name().is_none());
    }
}
