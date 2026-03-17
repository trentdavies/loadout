use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A registered source in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSource {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    pub plugins: Vec<RegisteredPlugin>,
    pub cache_path: PathBuf,
}

/// A plugin within a registered source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredPlugin {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub skills: Vec<RegisteredSkill>,
    pub path: PathBuf,
}

/// A skill within a registered plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSkill {
    pub name: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub path: PathBuf,
}

/// Provenance record for an installed skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub source: String,
    pub plugin: String,
    pub skill: String,
    /// Relative path from equip data dir to the skill's origin.
    pub origin: String,
}

/// The full registry index.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub sources: Vec<RegisteredSource>,

    /// Installed skills per agent: agent_name → skill_name → provenance.
    #[serde(default)]
    pub installed:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, InstalledSkill>>,
}
