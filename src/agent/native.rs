use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A plugin/skill installed natively by the agent (not by Equip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePlugin {
    /// Short name, e.g. "gopls-lsp".
    pub name: String,
    /// Full identifier as the agent knows it, e.g. "gopls-lsp@claude-plugins-official".
    pub full_id: String,
    /// Marketplace it came from, if known.
    pub marketplace: Option<String>,
    /// Agent-native install path, if known.
    pub path: Option<PathBuf>,
    /// How we discovered this entry.
    pub discovery_source: NativeDiscoverySource,
    /// Whether the plugin is enabled (None if unknown).
    pub enabled: Option<bool>,
}

/// Where a native plugin was discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeDiscoverySource {
    /// From installed_plugins.json (authoritative install record).
    InstalledPlugins,
    /// From settings.json enabledPlugins only (no install record found).
    EnabledPlugins,
    /// From the agent's skills/ directory (local, not tracked by Equip).
    LocalSkillsDir,
}

/// A marketplace known to the agent's native plugin system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeMarketplace {
    pub name: String,
    /// GitHub repo or URL, e.g. "anthropics/claude-plugins-official".
    pub repo: String,
    pub install_location: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Read-only detection of an agent's natively-installed plugins and marketplaces.
pub trait NativeDetector {
    /// All natively-installed plugins for this agent.
    fn native_plugins(&self, agent_path: &Path) -> Result<Vec<NativePlugin>>;

    /// Marketplaces the agent knows about.
    fn known_marketplaces(&self, agent_path: &Path) -> Result<Vec<NativeMarketplace>>;

    /// Check if a skill name collides with a native plugin.
    fn check_collision(&self, skill_name: &str, agent_path: &Path) -> Result<Option<NativePlugin>> {
        let plugins = self.native_plugins(agent_path)?;
        Ok(plugins.into_iter().find(|p| p.name == skill_name))
    }
}

/// Return the appropriate detector for an agent type, if one exists.
pub fn native_detector(agent_type: &str) -> Option<Box<dyn NativeDetector>> {
    match agent_type {
        "claude" => Some(Box::new(ClaudeDetector)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Claude implementation
// ---------------------------------------------------------------------------

struct ClaudeDetector;

// -- serde helpers for Claude's JSON files ----------------------------------

#[derive(Deserialize)]
struct InstalledPluginsFile {
    #[serde(default)]
    plugins: BTreeMap<String, Vec<InstalledPluginEntry>>,
}

#[derive(Deserialize)]
struct InstalledPluginEntry {
    #[serde(default, rename = "installPath")]
    install_path: Option<String>,
}

#[derive(Deserialize)]
struct SettingsFile {
    #[serde(default, rename = "enabledPlugins")]
    enabled_plugins: BTreeMap<String, bool>,
}

#[derive(Deserialize)]
struct KnownMarketplacesFile {
    #[serde(flatten)]
    marketplaces: BTreeMap<String, MarketplaceEntry>,
}

#[derive(Deserialize)]
struct MarketplaceEntry {
    source: MarketplaceSource,
    #[serde(default, rename = "installLocation")]
    install_location: Option<String>,
}

#[derive(Deserialize)]
struct MarketplaceSource {
    repo: String,
}

// -- helpers ----------------------------------------------------------------

/// Parse the `name@marketplace` convention. Returns (name, Some(marketplace))
/// or (full_id, None) if there is no `@`.
fn parse_plugin_id(full_id: &str) -> (String, Option<String>) {
    if let Some(idx) = full_id.find('@') {
        let name = full_id[..idx].to_string();
        let marketplace = full_id[idx + 1..].to_string();
        if marketplace.is_empty() {
            (name, None)
        } else {
            (name, Some(marketplace))
        }
    } else {
        (full_id.to_string(), None)
    }
}

/// Try to read and parse a JSON file. Returns Ok(None) for missing files,
/// logs a warning and returns Ok(None) for parse errors.
fn read_json_optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(val) => Ok(Some(val)),
            Err(e) => {
                eprintln!(
                    "  {} failed to parse {}: {}",
                    "warn:".yellow(),
                    path.display(),
                    e
                );
                Ok(None)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!(
                "  {} cannot read {}: {}",
                "warn:".yellow(),
                path.display(),
                e
            );
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

// Minimal colored-output helper so we don't depend on the full output module.
trait WarnColor {
    fn yellow(&self) -> String;
}
impl WarnColor for str {
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }
}

impl NativeDetector for ClaudeDetector {
    fn native_plugins(&self, agent_path: &Path) -> Result<Vec<NativePlugin>> {
        let mut plugins: Vec<NativePlugin> = Vec::new();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1. installed_plugins.json — authoritative install records
        let installed_path = agent_path.join("plugins/installed_plugins.json");
        if let Some(file) = read_json_optional::<InstalledPluginsFile>(&installed_path)? {
            for (full_id, entries) in &file.plugins {
                let (name, marketplace) = parse_plugin_id(full_id);
                let entry = entries.first();
                plugins.push(NativePlugin {
                    name,
                    full_id: full_id.clone(),
                    marketplace,
                    path: entry
                        .and_then(|e| e.install_path.as_ref())
                        .map(PathBuf::from),
                    discovery_source: NativeDiscoverySource::InstalledPlugins,
                    enabled: None, // filled from settings below
                });
                seen_ids.insert(full_id.clone());
            }
        }

        // 2. settings.json enabledPlugins — catches plugins not in installed_plugins
        let settings_path = agent_path.join("settings.json");
        if let Some(settings) = read_json_optional::<SettingsFile>(&settings_path)? {
            for (full_id, enabled) in &settings.enabled_plugins {
                if let Some(existing) = plugins.iter_mut().find(|p| p.full_id == *full_id) {
                    existing.enabled = Some(*enabled);
                } else {
                    let (name, marketplace) = parse_plugin_id(full_id);
                    plugins.push(NativePlugin {
                        name,
                        full_id: full_id.clone(),
                        marketplace,
                        path: None,
                        discovery_source: NativeDiscoverySource::EnabledPlugins,
                        enabled: Some(*enabled),
                    });
                    seen_ids.insert(full_id.clone());
                }
            }
        }

        // 3. skills/ directory — local skills not tracked by Equip
        let skills_dir = agent_path.join("skills");
        if skills_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                for entry in entries.flatten() {
                    if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        continue;
                    }
                    if !entry.path().join("SKILL.md").exists() {
                        continue;
                    }
                    let skill_name = entry.file_name().to_string_lossy().to_string();
                    // Skip if we already found this name from installed_plugins or enabled
                    if seen_ids.iter().any(|id| parse_plugin_id(id).0 == skill_name) {
                        continue;
                    }
                    plugins.push(NativePlugin {
                        name: skill_name.clone(),
                        full_id: skill_name,
                        marketplace: None,
                        path: Some(entry.path()),
                        discovery_source: NativeDiscoverySource::LocalSkillsDir,
                        enabled: None,
                    });
                }
            }
        }

        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(plugins)
    }

    fn known_marketplaces(&self, agent_path: &Path) -> Result<Vec<NativeMarketplace>> {
        let path = agent_path.join("plugins/known_marketplaces.json");
        let file = match read_json_optional::<KnownMarketplacesFile>(&path)? {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut marketplaces: Vec<NativeMarketplace> = file
            .marketplaces
            .into_iter()
            .map(|(name, entry)| NativeMarketplace {
                name,
                repo: entry.source.repo,
                install_location: entry.install_location.map(PathBuf::from),
            })
            .collect();
        marketplaces.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(marketplaces)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_claude_dir(tmp: &Path) {
        // installed_plugins.json
        let plugins_dir = tmp.join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(
            plugins_dir.join("installed_plugins.json"),
            r#"{
                "version": 2,
                "plugins": {
                    "gopls-lsp@claude-plugins-official": [{
                        "scope": "user",
                        "installPath": "/tmp/cache/gopls-lsp/1.0.0",
                        "version": "1.0.0"
                    }],
                    "rust-analyzer-lsp@claude-plugins-official": [{
                        "scope": "user",
                        "installPath": "/tmp/cache/rust-analyzer-lsp/1.0.0",
                        "version": "1.0.0"
                    }]
                }
            }"#,
        )
        .unwrap();

        // settings.json with enabled/disabled and one extra plugin
        fs::write(
            tmp.join("settings.json"),
            r#"{
                "enabledPlugins": {
                    "gopls-lsp@claude-plugins-official": true,
                    "rust-analyzer-lsp@claude-plugins-official": false,
                    "custom-tool@my-marketplace": true
                }
            }"#,
        )
        .unwrap();

        // known_marketplaces.json
        fs::write(
            plugins_dir.join("known_marketplaces.json"),
            r#"{
                "claude-plugins-official": {
                    "source": { "source": "github", "repo": "anthropics/claude-plugins-official" },
                    "installLocation": "/tmp/marketplaces/claude-plugins-official"
                }
            }"#,
        )
        .unwrap();

        // A local skill in skills/
        let skill_dir = tmp.join("skills/my-local-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-local-skill\n---\nHello").unwrap();
    }

    #[test]
    fn test_native_plugins_from_installed_and_settings() {
        let tmp = tempfile::tempdir().unwrap();
        setup_claude_dir(tmp.path());

        let detector = ClaudeDetector;
        let plugins = detector.native_plugins(tmp.path()).unwrap();

        // Should find: custom-tool, gopls-lsp, my-local-skill, rust-analyzer-lsp (sorted)
        let names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["custom-tool", "gopls-lsp", "my-local-skill", "rust-analyzer-lsp"]
        );

        // gopls-lsp: from InstalledPlugins, enabled = true
        let gopls = plugins.iter().find(|p| p.name == "gopls-lsp").unwrap();
        assert_eq!(gopls.discovery_source, NativeDiscoverySource::InstalledPlugins);
        assert_eq!(gopls.enabled, Some(true));
        assert_eq!(
            gopls.marketplace.as_deref(),
            Some("claude-plugins-official")
        );

        // rust-analyzer: from InstalledPlugins, enabled = false
        let ra = plugins.iter().find(|p| p.name == "rust-analyzer-lsp").unwrap();
        assert_eq!(ra.enabled, Some(false));

        // custom-tool: from EnabledPlugins only
        let custom = plugins.iter().find(|p| p.name == "custom-tool").unwrap();
        assert_eq!(custom.discovery_source, NativeDiscoverySource::EnabledPlugins);
        assert_eq!(custom.marketplace.as_deref(), Some("my-marketplace"));

        // my-local-skill: from LocalSkillsDir
        let local = plugins.iter().find(|p| p.name == "my-local-skill").unwrap();
        assert_eq!(local.discovery_source, NativeDiscoverySource::LocalSkillsDir);
    }

    #[test]
    fn test_known_marketplaces() {
        let tmp = tempfile::tempdir().unwrap();
        setup_claude_dir(tmp.path());

        let detector = ClaudeDetector;
        let marketplaces = detector.known_marketplaces(tmp.path()).unwrap();

        assert_eq!(marketplaces.len(), 1);
        assert_eq!(marketplaces[0].name, "claude-plugins-official");
        assert_eq!(marketplaces[0].repo, "anthropics/claude-plugins-official");
    }

    #[test]
    fn test_collision_check() {
        let tmp = tempfile::tempdir().unwrap();
        setup_claude_dir(tmp.path());

        let detector = ClaudeDetector;
        let collision = detector.check_collision("gopls-lsp", tmp.path()).unwrap();
        assert!(collision.is_some());
        assert_eq!(collision.unwrap().name, "gopls-lsp");

        let no_collision = detector
            .check_collision("nonexistent-skill", tmp.path())
            .unwrap();
        assert!(no_collision.is_none());
    }

    #[test]
    fn test_missing_files_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        // Empty directory — no JSON files at all

        let detector = ClaudeDetector;
        let plugins = detector.native_plugins(tmp.path()).unwrap();
        assert!(plugins.is_empty());

        let marketplaces = detector.known_marketplaces(tmp.path()).unwrap();
        assert!(marketplaces.is_empty());
    }

    #[test]
    fn test_parse_plugin_id() {
        assert_eq!(
            parse_plugin_id("gopls-lsp@claude-plugins-official"),
            ("gopls-lsp".to_string(), Some("claude-plugins-official".to_string()))
        );
        assert_eq!(
            parse_plugin_id("standalone-plugin"),
            ("standalone-plugin".to_string(), None)
        );
        assert_eq!(
            parse_plugin_id("trailing-at@"),
            ("trailing-at".to_string(), None)
        );
    }

    #[test]
    fn test_factory() {
        assert!(native_detector("claude").is_some());
        assert!(native_detector("codex").is_none());
        assert!(native_detector("cursor").is_none());
    }
}
