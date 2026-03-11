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

/// The skittle config directory (`$XDG_CONFIG_HOME/skittle` or `~/.config/skittle`).
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("skittle")
}

/// The skittle data directory (`$XDG_DATA_HOME/skittle` or `~/.local/share/skittle`).
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("skittle")
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
