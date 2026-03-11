use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Parsed source.toml manifest.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SourceManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub plugins: Option<Vec<String>>,
    pub assets: Option<Vec<String>>,
}

/// Parsed plugin.toml manifest.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub assets: Option<Vec<String>>,
}

// -- source.toml file format (supports [source] wrapper or flat) --

#[derive(Debug, serde::Deserialize)]
struct SourceManifestFile {
    source: Option<SourceManifestSection>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    plugins: Option<Vec<String>>,
    assets: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
struct SourceManifestSection {
    name: String,
    version: Option<String>,
    description: Option<String>,
    plugins: Option<Vec<String>>,
    assets: Option<Vec<String>>,
}

impl SourceManifestFile {
    fn into_manifest(self) -> Result<SourceManifest> {
        if let Some(s) = self.source {
            Ok(SourceManifest {
                name: s.name,
                version: s.version,
                description: s.description,
                plugins: s.plugins,
                assets: s.assets,
            })
        } else {
            let name = self.name
                .filter(|n| !n.is_empty())
                .ok_or_else(|| anyhow::anyhow!("source.toml: 'name' is required"))?;
            Ok(SourceManifest {
                name,
                version: self.version,
                description: self.description,
                plugins: self.plugins,
                assets: self.assets,
            })
        }
    }
}

// -- plugin.toml file format (supports [plugin] wrapper or flat) --

#[derive(Debug, serde::Deserialize)]
struct PluginManifestFile {
    plugin: Option<PluginManifestSection>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    assets: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
struct PluginManifestSection {
    name: String,
    version: Option<String>,
    description: Option<String>,
    assets: Option<Vec<String>>,
}

impl PluginManifestFile {
    fn into_manifest(self) -> Result<PluginManifest> {
        if let Some(p) = self.plugin {
            Ok(PluginManifest {
                name: p.name,
                version: p.version,
                description: p.description,
                assets: p.assets,
            })
        } else {
            let name = self.name
                .filter(|n| !n.is_empty())
                .ok_or_else(|| anyhow::anyhow!("plugin.toml: 'name' is required"))?;
            Ok(PluginManifest {
                name,
                version: self.version,
                description: self.description,
                assets: self.assets,
            })
        }
    }
}

/// Load and validate a source.toml file.
pub fn load_source_manifest(path: &Path) -> Result<SourceManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file: SourceManifestFile = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    file.into_manifest()
}

/// Load and validate a plugin.toml file.
pub fn load_plugin_manifest(path: &Path) -> Result<PluginManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file: PluginManifestFile = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    file.into_manifest()
}
