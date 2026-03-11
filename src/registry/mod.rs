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

impl Registry {
    /// Find a plugin by name across all sources.
    /// Returns (source_name, plugin) tuple.
    pub fn find_plugin(&self, name: &str) -> Option<(&str, &RegisteredPlugin)> {
        for src in &self.sources {
            for plugin in &src.plugins {
                if plugin.name == name {
                    return Some((&src.name, plugin));
                }
            }
        }
        None
    }

    /// Find a skill by identity string.
    /// Accepts `plugin/skill` or `source:plugin/skill`.
    /// Returns (source_name, plugin_name, skill) tuple.
    pub fn find_skill(&self, identity: &str) -> Result<(&str, &str, &RegisteredSkill)> {
        let (source_filter, plugin_name, skill_name) = parse_skill_identity(identity)?;

        let mut matches = Vec::new();

        for src in &self.sources {
            if let Some(ref filter) = source_filter {
                if src.name != *filter {
                    continue;
                }
            }
            for plugin in &src.plugins {
                if plugin.name != plugin_name {
                    continue;
                }
                for skill in &plugin.skills {
                    if skill.name == skill_name {
                        matches.push((&*src.name, &*plugin.name, skill));
                    }
                }
            }
        }

        match matches.len() {
            0 => anyhow::bail!("skill '{}' not found", identity),
            1 => Ok((matches[0].0, matches[0].1, matches[0].2)),
            _ => {
                let sources: Vec<String> = matches.iter()
                    .map(|(s, p, sk)| format!("  {}:{}/{}", s, p, sk.name))
                    .collect();
                anyhow::bail!(
                    "skill '{}' is ambiguous. Use fully qualified form:\n{}",
                    identity,
                    sources.join("\n")
                );
            }
        }
    }

    /// Iterate all (source_name, plugin, skill) triples.
    pub fn all_skills(&self) -> Vec<(&str, &RegisteredPlugin, &RegisteredSkill)> {
        let mut result = Vec::new();
        for src in &self.sources {
            for plugin in &src.plugins {
                for skill in &plugin.skills {
                    result.push((src.name.as_str(), plugin, skill));
                }
            }
        }
        result
    }

    /// Set the active bundle for a target.
    pub fn set_active_bundle(&mut self, target_name: &str, bundle_name: &str) {
        self.active_bundles.insert(target_name.to_string(), bundle_name.to_string());
    }

    /// Get the active bundle for a target.
    pub fn active_bundle(&self, target_name: &str) -> Option<&str> {
        self.active_bundles.get(target_name).map(|s| s.as_str())
    }

    /// Clear the active bundle for a target.
    pub fn clear_active_bundle(&mut self, target_name: &str) {
        self.active_bundles.remove(target_name);
    }
}

/// Parse a skill identity string into (optional_source, plugin, skill).
fn parse_skill_identity(identity: &str) -> Result<(Option<String>, String, String)> {
    // source:plugin/skill
    if let Some((source, rest)) = identity.split_once(':') {
        if let Some((plugin, skill)) = rest.split_once('/') {
            return Ok((Some(source.to_string()), plugin.to_string(), skill.to_string()));
        }
        anyhow::bail!("invalid skill identity '{}': expected source:plugin/skill", identity);
    }

    // plugin/skill
    if let Some((plugin, skill)) = identity.split_once('/') {
        return Ok((None, plugin.to_string(), skill.to_string()));
    }

    anyhow::bail!(
        "invalid skill identity '{}': expected plugin/skill or source:plugin/skill",
        identity
    );
}
