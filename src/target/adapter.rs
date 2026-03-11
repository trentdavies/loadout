use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use crate::config::{AdapterConfig, TargetConfig};
use crate::registry::RegisteredSkill;

/// Known built-in agent types.
const BUILTIN_AGENTS: &[&str] = &["claude", "codex", "cursor", "gemini", "vscode"];

/// Default directories to copy from a skill source.
const DEFAULT_COPY_DIRS: &[&str] = &["scripts", "references", "assets"];

/// Adapter for installing/uninstalling skills to a target.
pub struct Adapter {
    /// Template for skill directory: e.g., "skills/{name}" or "prompts/{name}".
    skill_dir_template: String,
    /// Filename for the skill file (always SKILL.md for agentskills format).
    skill_file: String,
    /// Which subdirectories to copy from the source skill.
    copy_dirs: Vec<String>,
    /// The format — only "agentskills" is supported.
    format: String,
}

impl Adapter {
    /// Install a skill to the target directory.
    pub fn install_skill(&self, skill: &RegisteredSkill, target_path: &Path) -> Result<()> {
        if self.format != "agentskills" {
            anyhow::bail!(
                "unsupported adapter format '{}'. Only 'agentskills' is supported.",
                self.format
            );
        }

        let dest_dir = self.skill_dest(target_path, &skill.name);
        fs::create_dir_all(&dest_dir)
            .with_context(|| format!("failed to create {}", dest_dir.display()))?;

        // Copy SKILL.md
        let src_skill_file = skill.path.join(&self.skill_file);
        if src_skill_file.exists() {
            fs::copy(&src_skill_file, dest_dir.join(&self.skill_file))
                .with_context(|| format!("failed to copy {}", self.skill_file))?;
        }

        // Copy configured subdirectories
        for dir_name in &self.copy_dirs {
            let src_dir = skill.path.join(dir_name);
            if src_dir.is_dir() {
                copy_dir_recursive(&src_dir, &dest_dir.join(dir_name))?;
            }
        }

        Ok(())
    }

    /// Uninstall a skill from the target directory.
    pub fn uninstall_skill(&self, skill_name: &str, target_path: &Path) -> Result<()> {
        let dest_dir = self.skill_dest(target_path, skill_name);
        if dest_dir.exists() {
            fs::remove_dir_all(&dest_dir)
                .with_context(|| format!("failed to remove {}", dest_dir.display()))?;
        }
        Ok(())
    }

    /// List installed skill names at the target.
    pub fn installed_skills(&self, target_path: &Path) -> Result<Vec<String>> {
        // Extract the parent dir from the template (e.g., "skills" from "skills/{name}")
        let base = self.skills_base_dir(target_path);
        if !base.is_dir() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();
        for entry in fs::read_dir(&base)?.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if entry.path().join(&self.skill_file).exists() {
                    names.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// Resolve the destination directory for a skill.
    fn skill_dest(&self, target_path: &Path, skill_name: &str) -> PathBuf {
        let resolved = self.skill_dir_template.replace("{name}", skill_name);
        target_path.join(resolved)
    }

    /// Get the base directory that contains skill subdirs.
    fn skills_base_dir(&self, target_path: &Path) -> PathBuf {
        // Template is like "skills/{name}" — strip the /{name} part
        let base = if let Some(idx) = self.skill_dir_template.find("/{name}") {
            &self.skill_dir_template[..idx]
        } else if let Some(idx) = self.skill_dir_template.find("{name}") {
            &self.skill_dir_template[..idx]
        } else {
            &self.skill_dir_template
        };
        target_path.join(base)
    }
}

/// Resolve the adapter for a target configuration.
/// Uses built-in adapters for known agents, custom TOML adapters otherwise.
pub fn resolve_adapter(
    target: &TargetConfig,
    custom_adapters: &std::collections::BTreeMap<String, AdapterConfig>,
) -> Result<Adapter> {
    // Check for custom adapter first (overrides built-in)
    if let Some(custom) = custom_adapters.get(&target.agent) {
        if custom.format != "agentskills" {
            anyhow::bail!(
                "unsupported adapter format '{}' for agent '{}'. Only 'agentskills' is supported.",
                custom.format,
                target.agent
            );
        }
        return Ok(Adapter {
            skill_dir_template: custom.skill_dir.clone(),
            skill_file: custom.skill_file.clone(),
            copy_dirs: custom.copy_dirs.clone(),
            format: custom.format.clone(),
        });
    }

    // Built-in adapters
    if BUILTIN_AGENTS.contains(&target.agent.as_str()) {
        return Ok(builtin_adapter());
    }

    let available: Vec<String> = BUILTIN_AGENTS.iter()
        .map(|s| s.to_string())
        .chain(custom_adapters.keys().cloned())
        .collect();

    anyhow::bail!(
        "no adapter for agent type '{}'. Available: {}",
        target.agent,
        available.join(", ")
    );
}

/// The standard built-in adapter (claude, codex, cursor all use the same layout).
fn builtin_adapter() -> Adapter {
    Adapter {
        skill_dir_template: "skills/{name}".to_string(),
        skill_file: "SKILL.md".to_string(),
        copy_dirs: DEFAULT_COPY_DIRS.iter().map(|s| s.to_string()).collect(),
        format: "agentskills".to_string(),
    }
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
