use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A registered source in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSource {
    pub name: String,
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
    pub path: PathBuf,
}

/// The full registry index.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub sources: Vec<RegisteredSource>,
}
