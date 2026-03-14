use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{AdapterConfig, TargetConfig};
use crate::registry::RegisteredSkill;

/// Status of a skill at a target relative to the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillStatus {
    /// Skill directory does not exist at the target.
    New,
    /// All files match byte-for-byte between source and target.
    Unchanged,
    /// At least one file differs between source and target.
    Changed,
}

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

    /// Compare a source skill against what's installed at the target.
    pub fn compare_skill(
        &self,
        skill: &RegisteredSkill,
        target_path: &Path,
    ) -> Result<SkillStatus> {
        let dest_dir = self.skill_dest(target_path, &skill.name);
        if !dest_dir.exists() {
            return Ok(SkillStatus::New);
        }

        // Compare SKILL.md
        let src_skill_file = skill.path.join(&self.skill_file);
        let dst_skill_file = dest_dir.join(&self.skill_file);
        if (src_skill_file.exists() || dst_skill_file.exists())
            && !files_equal(&src_skill_file, &dst_skill_file)?
        {
            return Ok(SkillStatus::Changed);
        }

        // Compare configured subdirectories
        for dir_name in &self.copy_dirs {
            let src_dir = skill.path.join(dir_name);
            let dst_dir = dest_dir.join(dir_name);
            if (src_dir.is_dir() || dst_dir.is_dir()) && !dirs_equal(&src_dir, &dst_dir)? {
                return Ok(SkillStatus::Changed);
            }
        }

        Ok(SkillStatus::Unchanged)
    }

    /// Collect all file paths within a skill directory (relative to skill root),
    /// along with their absolute paths on both sides. Used for diff display.
    pub fn skill_file_pairs(
        &self,
        skill: &RegisteredSkill,
        target_path: &Path,
    ) -> Result<Vec<(String, PathBuf, PathBuf)>> {
        let dest_dir = self.skill_dest(target_path, &skill.name);
        let mut pairs = Vec::new();

        // SKILL.md
        let src_file = skill.path.join(&self.skill_file);
        let dst_file = dest_dir.join(&self.skill_file);
        if src_file.exists() || dst_file.exists() {
            pairs.push((self.skill_file.clone(), src_file, dst_file));
        }

        // Subdirectories
        for dir_name in &self.copy_dirs {
            let src_dir = skill.path.join(dir_name);
            let dst_dir = dest_dir.join(dir_name);
            collect_file_pairs_recursive(&src_dir, &dst_dir, dir_name, &mut pairs)?;
        }

        Ok(pairs)
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
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && entry.path().join(&self.skill_file).exists()
            {
                names.push(entry.file_name().to_string_lossy().to_string());
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
        return Ok(builtin_adapter(&target.agent));
    }

    let available: Vec<String> = BUILTIN_AGENTS
        .iter()
        .map(|s| s.to_string())
        .chain(custom_adapters.keys().cloned())
        .collect();

    anyhow::bail!(
        "no adapter for agent type '{}'. Available: {}",
        target.agent,
        available.join(", ")
    );
}

/// Built-in adapter for known agent types.
/// Most agents store skills in `skills/{name}`, but cursor stores directly in `{name}`.
fn builtin_adapter(agent: &str) -> Adapter {
    let skill_dir_template = match agent {
        "cursor" => "{name}".to_string(),
        _ => "skills/{name}".to_string(),
    };
    Adapter {
        skill_dir_template,
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

/// Compare two files byte-for-byte. Returns false if either doesn't exist
/// or if contents differ.
fn files_equal(a: &Path, b: &Path) -> Result<bool> {
    match (a.exists(), b.exists()) {
        (false, false) => Ok(true),
        (true, false) | (false, true) => Ok(false),
        (true, true) => {
            let a_bytes = fs::read(a).with_context(|| format!("failed to read {}", a.display()))?;
            let b_bytes = fs::read(b).with_context(|| format!("failed to read {}", b.display()))?;
            Ok(a_bytes == b_bytes)
        }
    }
}

/// Recursively compare two directories. Returns false if any file differs,
/// is missing on one side, or directory structure differs.
fn dirs_equal(src: &Path, dst: &Path) -> Result<bool> {
    match (src.is_dir(), dst.is_dir()) {
        (false, false) => return Ok(true),
        (true, false) | (false, true) => return Ok(false),
        (true, true) => {}
    }

    // Collect all relative paths from both sides
    let mut src_files = std::collections::BTreeSet::new();
    collect_relative_paths(src, src, &mut src_files)?;
    let mut dst_files = std::collections::BTreeSet::new();
    collect_relative_paths(dst, dst, &mut dst_files)?;

    if src_files != dst_files {
        return Ok(false);
    }

    for rel in &src_files {
        if !files_equal(&src.join(rel), &dst.join(rel))? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Collect all file paths relative to `base` under `dir`.
fn collect_relative_paths(
    dir: &Path,
    base: &Path,
    out: &mut std::collections::BTreeSet<PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_paths(&path, base, out)?;
        } else {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            out.insert(rel);
        }
    }
    Ok(())
}

/// Collect (relative_label, src_path, dst_path) pairs for all files across
/// source and destination directories, recursively.
fn collect_file_pairs_recursive(
    src_dir: &Path,
    dst_dir: &Path,
    prefix: &str,
    pairs: &mut Vec<(String, PathBuf, PathBuf)>,
) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();

    // Files from source side
    if src_dir.is_dir() {
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let label = format!("{}/{}", prefix, name);
            let src_path = entry.path();
            let dst_path = dst_dir.join(&name);
            if src_path.is_dir() {
                collect_file_pairs_recursive(&src_path, &dst_path, &label, pairs)?;
            } else {
                pairs.push((label.clone(), src_path, dst_path));
            }
            seen.insert(name);
        }
    }

    // Files only on destination side
    if dst_dir.is_dir() {
        for entry in fs::read_dir(dst_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if seen.contains(&name) {
                continue;
            }
            let label = format!("{}/{}", prefix, name);
            let src_path = src_dir.join(&name);
            let dst_path = entry.path();
            if dst_path.is_dir() {
                collect_file_pairs_recursive(&src_path, &dst_path, &label, pairs)?;
            } else {
                pairs.push((label, src_path, dst_path));
            }
        }
    }

    Ok(())
}
