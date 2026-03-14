pub mod types;

pub use types::*;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the config file path.
/// Uses `--config` override if provided, otherwise `~/.local/share/skittle/skittle.toml`.
pub fn config_path(override_path: Option<&str>) -> PathBuf {
    if let Some(p) = override_path {
        return PathBuf::from(p);
    }
    data_dir().join("skittle.toml")
}

/// The skittle data directory. Everything lives here — config, registry, cached sources.
/// Respects `$XDG_DATA_HOME` first, then falls back to `~/.local/share/skittle`.
pub fn data_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("skittle");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".local")
        .join("share")
        .join("skittle")
}

/// The external source cache directory: `<data_dir>/external/`.
/// Creates the directory if it doesn't exist.
pub fn cache_dir() -> PathBuf {
    let dir = data_dir().join("external");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// The managed plugins directory: `<data_dir>/plugins/`.
pub fn plugins_dir() -> PathBuf {
    data_dir().join("plugins")
}

/// The skittle internals directory: `<data_dir>/.skittle/`.
pub fn internal_dir() -> PathBuf {
    data_dir().join(".skittle")
}

/// Load config from the resolved path.
/// Returns default Config if the file doesn't exist.
pub fn load(override_path: Option<&str>) -> Result<Config> {
    let path = config_path(override_path);
    load_from(&path)
}

/// Load config from a specific path.
pub fn load_from(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    Ok(config)
}

/// Save config to the resolved path, creating directories as needed.
pub fn save(config: &Config, override_path: Option<&str>) -> Result<()> {
    let path = config_path(override_path);
    save_to(config, &path)
}

/// Save config to a specific path.
pub fn save_to(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(path, content)
        .with_context(|| format!("failed to write config: {}", path.display()))?;
    Ok(())
}

/// The default config template written by `skittle init`.
pub const DEFAULT_CONFIG: &str = r#"# Skittle — Agent Skill Manager
# This file lives in ~/.local/share/skittle/ alongside your registry and cached sources.
# This directory can be a git repo for versioning your configuration.

# ─── Sources ────────────────────────────────────────────────────────────────
# Where skills come from. Add with: skittle add <url>
#
# [[source]]
# name = "anthropic-plugins"
# url = "https://github.com/anthropics/knowledge-work-plugins.git"
# type = "git"
#
# [[source]]
# name = "my-skills"
# url = "~/dev/my-skills"
# type = "local"
#
# [[source]]
# name = "team-tools"
# url = "git@github.com:myorg/agent-skills.git"
# type = "git"

# ─── Targets ────────────────────────────────────────────────────────────────
# Where skills get installed. Add with: skittle target add <agent> [path]
# Targets with sync = "auto" receive skills from `skittle install --all`.
#
# [[target]]
# name = "claude"
# agent = "claude"
# path = "~/.claude"
# scope = "machine"
# sync = "auto"
#
# [[target]]
# name = "codex"
# agent = "codex"
# path = "~/.codex"
# scope = "machine"
# sync = "auto"
#
# [[target]]
# name = "project-claude"
# agent = "claude"
# path = "./my-project/.claude"
# scope = "repo"
# sync = "explicit"

# ─── Bundles ────────────────────────────────────────────────────────────────
# Named groups of skills. Create with: skittle bundle create <name>
# Install with: skittle install --bundle <name>
# Swap between bundles: skittle bundle swap <from> <to> --force
#
# [bundle.work]
# skills = ["legal/contract-review", "legal/compliance", "sales/call-prep"]
#
# [bundle.personal]
# skills = ["productivity/daily-planner", "engineering/code-review"]
#
# [bundle.minimal]
# skills = ["productivity/daily-planner"]

# ─── Custom Adapters ───────────────────────────────────────────────────────
# Define how skills are installed for non-standard agents.
# Built-in adapters exist for: claude, codex, cursor, gemini, vscode.
#
# [adapter.my-agent]
# skill_dir = "prompts/{name}"
# skill_file = "SKILL.md"
# format = "agentskills"
# copy_dirs = ["scripts", "references"]
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn config_path_with_override() {
        let p = config_path(Some("/custom/path.toml"));
        assert_eq!(p, PathBuf::from("/custom/path.toml"));
    }

    #[test]
    fn config_path_default() {
        let p = config_path(None);
        assert!(p.to_string_lossy().contains("skittle"));
        assert!(p.to_string_lossy().ends_with("skittle.toml"));
    }

    #[test]
    fn config_lives_in_data_dir() {
        let config = config_path(None);
        let data = data_dir();
        assert!(config.starts_with(&data));
    }

    #[test]
    fn load_from_nonexistent_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");
        let config = load_from(&path).unwrap();
        assert!(config.source.is_empty());
        assert!(config.target.is_empty());
    }

    #[test]
    fn load_from_invalid_toml_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        fs::write(&path, "[invalid toml").unwrap();
        assert!(load_from(&path).is_err());
    }

    #[test]
    fn save_to_load_from_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("skittle.toml");

        let mut config = Config::default();
        config.source.push(SourceConfig {
            name: "test-src".to_string(),
            url: "/tmp/skills".to_string(),
            source_type: "local".to_string(),
            r#ref: None,
            mode: None,
        });
        config.target.push(TargetConfig {
            name: "test-tgt".to_string(),
            agent: "claude".to_string(),
            path: PathBuf::from("/tmp/claude"),
            scope: "machine".to_string(),
            sync: "auto".to_string(),
        });

        save_to(&config, &path).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.source.len(), 1);
        assert_eq!(loaded.source[0].name, "test-src");
        assert_eq!(loaded.target.len(), 1);
        assert_eq!(loaded.target[0].name, "test-tgt");
    }

    #[test]
    fn save_to_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("deep").join("nested").join("skittle.toml");
        let config = Config::default();
        save_to(&config, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn load_via_override() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("custom.toml");
        let config = Config::default();
        save_to(&config, &path).unwrap();

        let loaded = load(Some(path.to_str().unwrap())).unwrap();
        assert!(loaded.source.is_empty());
    }
}
