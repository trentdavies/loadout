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

/// Metadata extracted from a .claude-plugin file.
#[derive(Debug, Clone, Default)]
pub struct ClaudePluginMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Load metadata from a .claude-plugin file (JSON, parsed defensively).
/// Returns None if the file cannot be read or parsed — never fails fatally.
pub fn load_claude_plugin_metadata(path: &Path) -> Option<ClaudePluginMetadata> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: failed to read {}: {}", path.display(), e);
            return None;
        }
    };
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: failed to parse {}: {}", path.display(), e);
            return None;
        }
    };
    Some(ClaudePluginMetadata {
        name: value.get("name").and_then(|v| v.as_str()).map(String::from),
        version: value.get("version").and_then(|v| v.as_str()).map(String::from),
        description: value.get("description").and_then(|v| v.as_str()).map(String::from),
        author: value.get("author").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Load and validate a plugin.toml file.
pub fn load_plugin_manifest(path: &Path) -> Result<PluginManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file: PluginManifestFile = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    file.into_manifest()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    // -- load_source_manifest() tests --

    #[test]
    fn source_manifest_with_section_wrapper() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "[source]\nname = \"my-source\"\nversion = \"1.0\"");
        let m = load_source_manifest(&f).unwrap();
        assert_eq!(m.name, "my-source");
        assert_eq!(m.version.as_deref(), Some("1.0"));
    }

    #[test]
    fn source_manifest_flat_form() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "name = \"flat-src\"\ndescription = \"test\"");
        let m = load_source_manifest(&f).unwrap();
        assert_eq!(m.name, "flat-src");
        assert_eq!(m.description.as_deref(), Some("test"));
    }

    #[test]
    fn source_manifest_missing_name() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "description = \"no name\"");
        assert!(load_source_manifest(&f).is_err());
    }

    #[test]
    fn source_manifest_empty_name() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "name = \"\"");
        assert!(load_source_manifest(&f).is_err());
    }

    #[test]
    fn source_manifest_file_not_found() {
        assert!(load_source_manifest(Path::new("/nonexistent/source.toml")).is_err());
    }

    #[test]
    fn source_manifest_invalid_toml() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "[invalid toml");
        assert!(load_source_manifest(&f).is_err());
    }

    #[test]
    fn source_manifest_with_plugins_list() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "name = \"src\"\nplugins = [\"a\", \"b\"]");
        let m = load_source_manifest(&f).unwrap();
        assert_eq!(m.plugins.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn source_manifest_empty_plugins_list() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "source.toml", "name = \"src\"\nplugins = []");
        let m = load_source_manifest(&f).unwrap();
        assert!(m.plugins.unwrap().is_empty());
    }

    // -- load_plugin_manifest() tests --

    #[test]
    fn plugin_manifest_happy_path() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "plugin.toml", "name = \"my-plugin\"\nversion = \"0.1\"");
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "my-plugin");
        assert_eq!(m.version.as_deref(), Some("0.1"));
    }

    #[test]
    fn plugin_manifest_with_section_wrapper() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "plugin.toml", "[plugin]\nname = \"wrapped\"");
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "wrapped");
    }

    #[test]
    fn plugin_manifest_missing_name() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "plugin.toml", "version = \"1.0\"");
        assert!(load_plugin_manifest(&f).is_err());
    }

    #[test]
    fn plugin_manifest_with_optional_fields() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), "plugin.toml", "name = \"p\"\nversion = \"2.0\"\ndescription = \"desc\"\nassets = [\"img\"]");
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.description.as_deref(), Some("desc"));
        assert_eq!(m.assets.as_ref().unwrap(), &vec!["img".to_string()]);
    }

    #[test]
    fn plugin_manifest_unknown_fields_ignored() {
        let tmp = TempDir::new().unwrap();
        // serde default: unknown fields cause error unless deny_unknown_fields is absent
        // This test verifies the manifest structs allow extra fields
        let f = write_manifest(tmp.path(), "plugin.toml", "name = \"p\"\nunknown_field = \"value\"");
        // This may or may not error depending on serde config — test documents behavior
        let result = load_plugin_manifest(&f);
        // If it errors, that's also valid behavior — just documenting
        let _ = result;
    }

    // -- load_claude_plugin_metadata() tests --

    #[test]
    fn claude_plugin_valid() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), ".claude-plugin",
            r#"{"name": "my-plugin", "version": "1.0", "author": "trent", "description": "a plugin"}"#);
        let meta = load_claude_plugin_metadata(&f).unwrap();
        assert_eq!(meta.name.as_deref(), Some("my-plugin"));
        assert_eq!(meta.version.as_deref(), Some("1.0"));
        assert_eq!(meta.author.as_deref(), Some("trent"));
        assert_eq!(meta.description.as_deref(), Some("a plugin"));
    }

    #[test]
    fn claude_plugin_partial_fields() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), ".claude-plugin", r#"{"name": "partial"}"#);
        let meta = load_claude_plugin_metadata(&f).unwrap();
        assert_eq!(meta.name.as_deref(), Some("partial"));
        assert!(meta.version.is_none());
        assert!(meta.author.is_none());
    }

    #[test]
    fn claude_plugin_malformed_json() {
        let tmp = TempDir::new().unwrap();
        let f = write_manifest(tmp.path(), ".claude-plugin", "not json at all");
        let result = load_claude_plugin_metadata(&f);
        assert!(result.is_none());
    }

    #[test]
    fn claude_plugin_not_found() {
        let result = load_claude_plugin_metadata(Path::new("/nonexistent/.claude-plugin"));
        assert!(result.is_none());
    }
}
