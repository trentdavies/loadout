use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Parsed marketplace.json — a multi-plugin source.
/// Found at `.claude-plugin/marketplace.json`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePlugin>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplaceOwner {
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub source: String,
    pub description: Option<String>,
    pub author: Option<MarketplaceAuthor>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplaceAuthor {
    pub name: Option<String>,
}

/// Parsed plugin.json — a single plugin.
/// Found at `.claude-plugin/plugin.json`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<PluginAuthor>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginAuthor {
    pub name: Option<String>,
}

/// Load and validate a marketplace.json file.
pub fn load_marketplace(path: &Path) -> Result<MarketplaceManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: MarketplaceManifest = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate a plugin.json file.
pub fn load_plugin_manifest(path: &Path) -> Result<PluginManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: PluginManifest = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_json(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    // -- marketplace.json tests --

    #[test]
    fn marketplace_valid() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "marketplace.json", r#"{
            "name": "my-marketplace",
            "owner": {"name": "Anthropic"},
            "plugins": [
                {"name": "legal", "source": "./legal", "description": "Legal tools"},
                {"name": "sales", "source": "./sales"}
            ]
        }"#);
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.name, "my-marketplace");
        assert_eq!(m.owner.unwrap().name.as_deref(), Some("Anthropic"));
        assert_eq!(m.plugins.len(), 2);
        assert_eq!(m.plugins[0].name, "legal");
        assert_eq!(m.plugins[0].source, "./legal");
        assert_eq!(m.plugins[0].description.as_deref(), Some("Legal tools"));
    }

    #[test]
    fn marketplace_with_plugin_authors() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "marketplace.json", r#"{
            "name": "mkt",
            "plugins": [
                {"name": "slack", "source": "./slack", "author": {"name": "Salesforce"}}
            ]
        }"#);
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.plugins[0].author.as_ref().unwrap().name.as_deref(), Some("Salesforce"));
    }

    #[test]
    fn marketplace_missing_name_errors() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "marketplace.json", r#"{"plugins": []}"#);
        assert!(load_marketplace(&f).is_err());
    }

    #[test]
    fn marketplace_not_found_errors() {
        assert!(load_marketplace(Path::new("/nonexistent/marketplace.json")).is_err());
    }

    #[test]
    fn marketplace_invalid_json_errors() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "marketplace.json", "not json");
        assert!(load_marketplace(&f).is_err());
    }

    // -- plugin.json tests --

    #[test]
    fn plugin_json_valid() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "plugin.json", r#"{
            "name": "legal",
            "version": "1.1.0",
            "description": "Legal tools",
            "author": {"name": "Anthropic"}
        }"#);
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "legal");
        assert_eq!(m.version.as_deref(), Some("1.1.0"));
        assert_eq!(m.description.as_deref(), Some("Legal tools"));
        assert_eq!(m.author.unwrap().name.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn plugin_json_minimal() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "plugin.json", r#"{"name": "minimal"}"#);
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "minimal");
        assert!(m.version.is_none());
        assert!(m.description.is_none());
        assert!(m.author.is_none());
    }

    #[test]
    fn plugin_json_missing_name_errors() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "plugin.json", r#"{"version": "1.0"}"#);
        assert!(load_plugin_manifest(&f).is_err());
    }

    #[test]
    fn plugin_json_not_found_errors() {
        assert!(load_plugin_manifest(Path::new("/nonexistent/plugin.json")).is_err());
    }

    #[test]
    fn plugin_json_extra_fields_ignored() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(tmp.path(), "plugin.json", r#"{"name": "p", "unknown_field": true}"#);
        // serde should ignore unknown fields by default
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "p");
    }
}
