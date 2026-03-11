use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Top-level skittle configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub source: Vec<SourceConfig>,

    #[serde(default)]
    pub target: Vec<TargetConfig>,

    #[serde(default)]
    pub adapter: BTreeMap<String, AdapterConfig>,

    #[serde(default)]
    pub bundle: BTreeMap<String, BundleConfig>,
}

/// A skill source (local path, git repo, URL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub url: String,

    #[serde(rename = "type", default = "default_source_type")]
    pub source_type: String,
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
