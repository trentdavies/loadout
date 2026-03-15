use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
    #[serde(deserialize_with = "deserialize_plugin_source")]
    pub source: PluginSource,
    pub description: Option<String>,
    pub author: Option<MarketplaceAuthor>,
    /// Explicit skill paths belonging to this plugin (e.g. `["./skills/xlsx"]`).
    /// When present, only these skills are included in the plugin.
    pub skills: Option<Vec<String>>,
}

/// A marketplace plugin source — either a local path or an external reference.
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Local path relative to the marketplace root (e.g. `"./plugins/legal"`).
    Local(String),
    /// External URL-based source — not resolvable from the local clone.
    External { url: String, path: Option<String> },
}

impl PluginSource {
    /// Returns the local path if this is a local source, or None for external sources.
    pub fn local_path(&self) -> Option<&str> {
        match self {
            PluginSource::Local(p) => Some(p),
            PluginSource::External { .. } => None,
        }
    }
}

/// Custom deserializer for `source` field that accepts both string and object formats.
///
/// String format: `"./plugins/legal"` → `PluginSource::Local`
/// Object format: `{"source": "url", "url": "..."}` → `PluginSource::External`
/// Object format: `{"source": "git-subdir", "url": "...", "path": "..."}` → `PluginSource::External`
fn deserialize_plugin_source<'de, D>(deserializer: D) -> std::result::Result<PluginSource, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct SourceVisitor;

    impl<'de> de::Visitor<'de> for SourceVisitor {
        type Value = PluginSource;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string path or an object with source type and url")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<PluginSource, E> {
            Ok(PluginSource::Local(v.to_string()))
        }

        fn visit_map<M: de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> std::result::Result<PluginSource, M::Error> {
            let mut url: Option<String> = None;
            let mut path: Option<String> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "url" => url = Some(map.next_value()?),
                    "path" => path = Some(map.next_value()?),
                    _ => {
                        let _ = map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
            }

            let url = url.unwrap_or_default();
            Ok(PluginSource::External { url, path })
        }
    }

    deserializer.deserialize_any(SourceVisitor)
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
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: MarketplaceManifest = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate a plugin.json file.
pub fn load_plugin_manifest(path: &Path) -> Result<PluginManifest> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
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
        let f = write_json(
            tmp.path(),
            "marketplace.json",
            r#"{
            "name": "my-marketplace",
            "owner": {"name": "Anthropic"},
            "plugins": [
                {"name": "legal", "source": "./legal", "description": "Legal tools"},
                {"name": "sales", "source": "./sales"}
            ]
        }"#,
        );
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.name, "my-marketplace");
        assert_eq!(m.owner.unwrap().name.as_deref(), Some("Anthropic"));
        assert_eq!(m.plugins.len(), 2);
        assert_eq!(m.plugins[0].name, "legal");
        assert_eq!(m.plugins[0].source.local_path(), Some("./legal"));
        assert_eq!(m.plugins[0].description.as_deref(), Some("Legal tools"));
    }

    #[test]
    fn marketplace_with_plugin_authors() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(
            tmp.path(),
            "marketplace.json",
            r#"{
            "name": "mkt",
            "plugins": [
                {"name": "slack", "source": "./slack", "author": {"name": "Salesforce"}}
            ]
        }"#,
        );
        let m = load_marketplace(&f).unwrap();
        assert_eq!(
            m.plugins[0].author.as_ref().unwrap().name.as_deref(),
            Some("Salesforce")
        );
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

    #[test]
    fn marketplace_object_source_url() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(
            tmp.path(),
            "marketplace.json",
            r#"{
            "name": "mkt",
            "plugins": [
                {
                    "name": "atlassian",
                    "source": {"source": "url", "url": "https://github.com/atlassian/mcp.git"}
                }
            ]
        }"#,
        );
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.plugins.len(), 1);
        assert_eq!(m.plugins[0].name, "atlassian");
        assert!(m.plugins[0].source.local_path().is_none());
        match &m.plugins[0].source {
            PluginSource::External { url, path } => {
                assert_eq!(url, "https://github.com/atlassian/mcp.git");
                assert!(path.is_none());
            }
            _ => panic!("expected External"),
        }
    }

    #[test]
    fn marketplace_object_source_git_subdir() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(
            tmp.path(),
            "marketplace.json",
            r#"{
            "name": "mkt",
            "plugins": [
                {
                    "name": "railway",
                    "source": {
                        "source": "git-subdir",
                        "url": "railwayapp/railway-skills",
                        "path": "plugins/railway",
                        "ref": "main",
                        "sha": "abc123"
                    }
                }
            ]
        }"#,
        );
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.plugins.len(), 1);
        match &m.plugins[0].source {
            PluginSource::External { url, path } => {
                assert_eq!(url, "railwayapp/railway-skills");
                assert_eq!(path.as_deref(), Some("plugins/railway"));
            }
            _ => panic!("expected External"),
        }
    }

    #[test]
    fn marketplace_mixed_sources() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(
            tmp.path(),
            "marketplace.json",
            r#"{
            "name": "mkt",
            "plugins": [
                {"name": "local-plugin", "source": "./local-plugin"},
                {"name": "external", "source": {"source": "url", "url": "https://example.com/repo.git"}}
            ]
        }"#,
        );
        let m = load_marketplace(&f).unwrap();
        assert_eq!(m.plugins.len(), 2);
        assert_eq!(m.plugins[0].source.local_path(), Some("./local-plugin"));
        assert!(m.plugins[1].source.local_path().is_none());
    }

    // -- plugin.json tests --

    #[test]
    fn plugin_json_valid() {
        let tmp = TempDir::new().unwrap();
        let f = write_json(
            tmp.path(),
            "plugin.json",
            r#"{
            "name": "legal",
            "version": "1.1.0",
            "description": "Legal tools",
            "author": {"name": "Anthropic"}
        }"#,
        );
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
        let f = write_json(
            tmp.path(),
            "plugin.json",
            r#"{"name": "p", "unknown_field": true}"#,
        );
        let m = load_plugin_manifest(&f).unwrap();
        assert_eq!(m.name, "p");
    }
}
