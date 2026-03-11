pub mod types;

pub use types::*;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Load the registry index from disk.
pub fn load_registry(data_dir: &Path) -> Result<Registry> {
    let path = data_dir.join("registry.json");
    if !path.exists() {
        return Ok(Registry::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let registry: Registry = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(registry)
}

/// Save the registry index to disk.
pub fn save_registry(registry: &Registry, data_dir: &Path) -> Result<()> {
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join("registry.json");
    let content = serde_json::to_string_pretty(registry)
        .context("failed to serialize registry")?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
