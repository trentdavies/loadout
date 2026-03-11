pub mod types;

pub use types::*;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the config file path.
/// Uses `--config` override if provided, otherwise XDG default.
pub fn config_path(override_path: Option<&str>) -> PathBuf {
    if let Some(p) = override_path {
        return PathBuf::from(p);
    }
    config_dir().join("config.toml")
}

/// The skittle config directory.
/// Respects `$XDG_CONFIG_HOME` first, then falls back to platform default.
pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("skittle");
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("skittle")
}

/// The skittle data directory.
/// Respects `$XDG_DATA_HOME` first, then falls back to platform default.
pub fn data_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("skittle");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("skittle")
}

/// The skittle source cache directory: `$XDG_DATA_HOME/skittle/sources/`.
/// Creates the directory if it doesn't exist.
pub fn cache_dir() -> PathBuf {
    let dir = data_dir().join("sources");
    let _ = std::fs::create_dir_all(&dir);
    dir
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
            .with_context(|| format!("failed to create config directory: {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config)
        .context("failed to serialize config")?;
    fs::write(path, content)
        .with_context(|| format!("failed to write config: {}", path.display()))?;
    Ok(())
}

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
        assert!(p.to_string_lossy().ends_with("config.toml"));
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
        let path = tmp.path().join("config.toml");

        let mut config = Config::default();
        config.source.push(SourceConfig {
            name: "test-src".to_string(),
            url: "/tmp/skills".to_string(),
            source_type: "local".to_string(),
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
        let path = tmp.path().join("deep").join("nested").join("config.toml");
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
