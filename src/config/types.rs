use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Top-level loadout configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub source: Vec<SourceConfig>,

    #[serde(default)]
    pub target: Vec<TargetConfig>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub adapter: BTreeMap<String, AdapterConfig>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bundle: BTreeMap<String, BundleConfig>,
}

/// A skill source (local path, git repo, URL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub url: String,

    #[serde(rename = "type", default = "default_source_type")]
    pub source_type: String,

    /// Pin to a specific git ref (tag, branch, or commit SHA). Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,

    /// Fetch mode for local sources: "symlink" or "copy". Omitted = copy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

fn default_source_type() -> String {
    "local".to_string()
}

/// An install target (agent installation directory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub name: String,
    pub agent: String,
    pub path: PathBuf,

    #[serde(default = "default_scope")]
    pub scope: String,

    #[serde(default = "default_sync")]
    pub sync: String,
}

fn default_scope() -> String {
    "machine".to_string()
}

fn default_sync() -> String {
    "auto".to_string()
}

/// Custom target adapter defined in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub skill_dir: String,
    pub skill_file: String,

    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default)]
    pub copy_dirs: Vec<String>,
}

fn default_format() -> String {
    "agentskills".to_string()
}

/// A named bundle of skills.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleConfig {
    #[serde(default)]
    pub skills: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_deserialize_empty() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.source.is_empty());
        assert!(config.target.is_empty());
        assert!(config.adapter.is_empty());
        assert!(config.bundle.is_empty());
    }

    #[test]
    fn config_deserialize_source() {
        let toml = r#"
[[source]]
name = "my-src"
url = "/tmp/skills"
type = "local"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.source.len(), 1);
        assert_eq!(config.source[0].name, "my-src");
        assert_eq!(config.source[0].source_type, "local");
    }

    #[test]
    fn config_deserialize_target_defaults() {
        let toml = r#"
[[target]]
name = "test"
agent = "claude"
path = "/tmp/claude"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.target[0].scope, "machine");
        assert_eq!(config.target[0].sync, "auto");
    }

    #[test]
    fn config_deserialize_adapter() {
        let toml = r#"
[adapter.myagent]
skill_dir = "prompts/{name}"
skill_file = "SKILL.md"
format = "agentskills"
copy_dirs = ["scripts"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.adapter.contains_key("myagent"));
        assert_eq!(config.adapter["myagent"].skill_dir, "prompts/{name}");
    }

    #[test]
    fn config_deserialize_bundle() {
        let toml = r#"
[bundle.dev]
skills = ["plugin/skill-a", "plugin/skill-b"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.bundle["dev"].skills.len(), 2);
    }

    #[test]
    fn config_serialize_roundtrip() {
        let mut config = Config::default();
        config.source.push(SourceConfig {
            name: "s".to_string(),
            url: "/tmp".to_string(),
            source_type: "local".to_string(),
            r#ref: None,
            mode: None,
        });
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.source[0].name, "s");
    }

    #[test]
    fn adapter_format_defaults_to_agentskills() {
        let toml = r#"
[adapter.custom]
skill_dir = "skills/{name}"
skill_file = "SKILL.md"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.adapter["custom"].format, "agentskills");
    }

    #[test]
    fn invalid_toml_deserialization_error() {
        let result = toml::from_str::<Config>("[invalid toml");
        assert!(result.is_err());
    }

    #[test]
    fn source_type_defaults_to_local() {
        let toml = r#"
[[source]]
name = "s"
url = "/tmp"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.source[0].source_type, "local");
    }
}
