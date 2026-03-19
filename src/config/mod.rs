pub mod types;

pub use types::*;

use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the config file path.
/// Uses `--config` override if provided, otherwise `~/.local/share/equip/equip.toml`.
pub fn config_path(override_path: Option<&str>) -> PathBuf {
    if let Some(p) = override_path {
        return PathBuf::from(p);
    }
    data_dir().join("equip.toml")
}

/// The equip data directory. Everything lives here — config, registry, cached sources.
/// Respects `$XDG_DATA_HOME` first, then falls back to `~/.local/share/equip`.
pub fn data_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("equip");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".local")
        .join("share")
        .join("equip")
}

/// The external source cache directory: `<data_dir>/external/`.
/// Creates the directory if it doesn't exist.
pub fn external_sources_dir() -> PathBuf {
    let dir = data_dir().join("external");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Backward-compatible alias for the external source cache directory.
pub fn cache_dir() -> PathBuf {
    external_sources_dir()
}

/// The repo-local source directory: the data dir root.
/// Managed local plugins live as direct children of the data dir
/// (e.g. `<data_dir>/local/`).
pub fn plugins_dir() -> PathBuf {
    data_dir()
}

/// Resolve the root directory for a given source residence.
pub fn source_dir(residence: SourceResidence) -> PathBuf {
    match residence {
        SourceResidence::External => external_sources_dir(),
        SourceResidence::Local => plugins_dir(),
    }
}

/// The equip internals directory: `<data_dir>/.equip/`.
pub fn internal_dir() -> PathBuf {
    data_dir().join(".equip")
}

/// Load config from the resolved path.
/// Returns default Config if the file doesn't exist.
pub fn load(override_path: Option<&str>) -> Result<Config> {
    let path = config_path(override_path);
    load_from(&path)
}

/// Load config from a specific path.
pub fn load_from(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    validate(&config, path)?;
    Ok(config)
}

/// Validate config: enforce kebab-case IDs and global uniqueness across sources and agents.
fn validate(config: &Config, path: &Path) -> Result<()> {
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert("local".to_string()); // reserved

    for source in &config.source {
        if !crate::source::detect::is_kebab_case(&source.id) {
            bail!(
                "source id '{}' is not valid kebab-case (lowercase letters, digits, hyphens)",
                source.id
            );
        }
        if !seen.insert(source.id.clone()) {
            bail!(
                "duplicate id '{}': source and agent IDs must be globally unique (repair in {})",
                source.id,
                path.display()
            );
        }
    }
    for agent in &config.agent {
        if !crate::source::detect::is_kebab_case(&agent.id) {
            bail!(
                "agent id '{}' is not valid kebab-case (lowercase letters, digits, hyphens)",
                agent.id
            );
        }
        if !seen.insert(agent.id.clone()) {
            bail!(
                "duplicate id '{}': source and agent IDs must be globally unique (repair in {})",
                agent.id,
                path.display()
            );
        }
    }
    Ok(())
}

/// Save config to the resolved path, creating directories as needed.
pub fn save(config: &Config, override_path: Option<&str>) -> Result<()> {
    let path = config_path(override_path);
    save_to(config, &path)
}

/// Save config to a specific path.
pub fn save_to(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(path, content)
        .with_context(|| format!("failed to write config: {}", path.display()))?;
    Ok(())
}

/// The default config template written by `equip init`.
pub const DEFAULT_CONFIG: &str = r#"# Equip — Agent Skill Manager
# This file lives in ~/.local/share/equip/ alongside your registry and cached sources.
# This directory can be a git repo for versioning your configuration.

# ─── Sources ────────────────────────────────────────────────────────────────
# Where skills come from.
#
# CLI:
#   equip add <url>                          # add a git or local source
#   equip add <url> --ref v1.2               # pin to a tag/branch/SHA
#   equip add ~/dev/my-skills --symlink      # local source via symlink
#   equip source remove <name> --force       # remove a source
#   equip source update [name]               # fetch latest from remote
#   equip source list                        # list all sources
#   equip list --external                    # compatibility alias for source list
#
# [[source]]
# id = "anthropic-plugins"
# url = "https://github.com/anthropics/knowledge-work-plugins.git"
# type = "git"
#
# [[source]]
# id = "my-skills"
# url = "~/dev/my-skills"
# type = "local"
#
# [[source]]
# id = "team-tools"
# url = "git@github.com:myorg/agent-skills.git"
# type = "git"

# ─── Agents ─────────────────────────────────────────────────────────────────
# Where skills get installed. Agents with sync = "auto" receive skills
# from `equip install --all`.
#
# CLI:
#   equip agent add claude                   # add an agent (auto-detects path)
#   equip agent add claude ./project/.claude --scope repo
#   equip agent remove <name> --force        # remove an agent
#   equip agent list                         # list all agents
#   equip agent detect                       # auto-detect installed agents
#
# [[agent]]
# id = "claude"
# type = "claude"
# path = "~/.claude"
# scope = "machine"
# sync = "auto"
#
# [[agent]]
# id = "codex"
# type = "codex"
# path = "~/.codex"
# scope = "machine"
# sync = "auto"
#
# [[agent]]
# id = "project-claude"
# type = "claude"
# path = "./my-project/.claude"
# scope = "repo"
# sync = "explicit"

# ─── Kits ──────────────────────────────────────────────────────────────────
# Named groups of skills you can equip/unequip together.
#
# CLI:
#   equip kit create <name> [skills...]      # create a kit, optionally with skills
#   equip kit add <name> <skills...>         # add skills to a kit
#   equip kit drop <name> <skills...>        # remove skills from a kit
#   equip kit list                           # list all kits
#   equip kit delete <name> --force          # delete a kit
#
#   equip agent equip -k <kit> --all         # equip a kit on all agents
#   equip agent unequip -k <kit> --all       # unequip a kit from all agents
#
# Example: context-switch between work and personal skill sets:
#
# [kit.work]
# skills = ["legal/contract-review", "legal/compliance", "sales/call-prep"]
#
# [kit.personal]
# skills = ["productivity/daily-planner", "engineering/code-review"]
#
# [kit.minimal]
# skills = ["productivity/daily-planner"]
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn config_path_with_override() {
        let p = config_path(Some("/custom/path.toml"));
        assert_eq!(p, PathBuf::from("/custom/path.toml"));
    }

    #[test]
    fn config_path_default() {
        let p = config_path(None);
        assert!(p.to_string_lossy().contains("equip"));
        assert!(p.to_string_lossy().ends_with("equip.toml"));
    }

    #[test]
    fn config_lives_in_data_dir() {
        let config = config_path(None);
        let data = data_dir();
        assert!(config.starts_with(&data));
    }

    #[test]
    fn load_from_nonexistent_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");
        let config = load_from(&path).unwrap();
        assert!(config.source.is_empty());
        assert!(config.agent.is_empty());
    }

    #[test]
    fn load_from_invalid_toml_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        fs::write(&path, "[invalid toml").unwrap();
        assert!(load_from(&path).is_err());
    }

    #[test]
    fn save_to_load_from_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("equip.toml");

        let mut config = Config::default();
        config.source.push(SourceConfig {
            id: "test-src".to_string(),
            url: "/tmp/skills".to_string(),
            source_type: "local".to_string(),
            r#ref: None,
            mode: None,
            residence: SourceResidence::External,
        });
        config.agent.push(AgentConfig {
            id: "test-tgt".to_string(),
            agent_type: "claude".to_string(),
            path: PathBuf::from("/tmp/claude"),
            scope: "machine".to_string(),
            sync: "auto".to_string(),
        });

        save_to(&config, &path).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.source.len(), 1);
        assert_eq!(loaded.source[0].id, "test-src");
        assert_eq!(loaded.agent.len(), 1);
        assert_eq!(loaded.agent[0].id, "test-tgt");
    }

    #[test]
    fn save_to_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("deep").join("nested").join("equip.toml");
        let config = Config::default();
        save_to(&config, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn load_via_override() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("custom.toml");
        let config = Config::default();
        save_to(&config, &path).unwrap();

        let loaded = load(Some(path.to_str().unwrap())).unwrap();
        assert!(loaded.source.is_empty());
    }

    #[test]
    fn backward_compat_name_alias_loads_into_id() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("compat.toml");
        fs::write(
            &path,
            r#"
[[source]]
name = "old-style"
url = "/tmp/skills"
type = "local"

[[agent]]
name = "old-agent"
type = "claude"
path = "/tmp/.claude"
"#,
        )
        .unwrap();
        let config = load_from(&path).unwrap();
        assert_eq!(config.source[0].id, "old-style");
        assert_eq!(config.agent[0].id, "old-agent");
    }

    #[test]
    fn validate_duplicate_source_ids_rejected() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("dup.toml");
        fs::write(
            &path,
            r#"
[[source]]
id = "dup"
url = "/tmp/a"
type = "local"

[[source]]
id = "dup"
url = "/tmp/b"
type = "local"
"#,
        )
        .unwrap();
        let err = load_from(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate id 'dup'"));
        assert!(msg.contains("repair in"));
    }

    #[test]
    fn validate_source_id_collides_with_agent_id() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("collision.toml");
        fs::write(
            &path,
            r#"
[[source]]
id = "shared-name"
url = "/tmp/skills"
type = "local"

[[agent]]
id = "shared-name"
type = "claude"
path = "/tmp/.claude"
"#,
        )
        .unwrap();
        let err = load_from(&path).unwrap_err();
        assert!(err.to_string().contains("duplicate id 'shared-name'"));
    }

    #[test]
    fn validate_local_reserved_as_source_id() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("reserved.toml");
        fs::write(
            &path,
            r#"
[[source]]
id = "local"
url = "/tmp/skills"
type = "local"
"#,
        )
        .unwrap();
        let err = load_from(&path).unwrap_err();
        assert!(err.to_string().contains("duplicate id 'local'"));
    }

    #[test]
    fn validate_non_kebab_case_source_id_rejected() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad-case.toml");
        fs::write(
            &path,
            r#"
[[source]]
id = "My_Source"
url = "/tmp/skills"
type = "local"
"#,
        )
        .unwrap();
        let err = load_from(&path).unwrap_err();
        assert!(err.to_string().contains("not valid kebab-case"));
    }

    #[test]
    fn validate_non_kebab_case_agent_id_rejected() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad-agent.toml");
        fs::write(
            &path,
            r#"
[[agent]]
id = "MyAgent"
type = "claude"
path = "/tmp/.claude"
"#,
        )
        .unwrap();
        let err = load_from(&path).unwrap_err();
        assert!(err.to_string().contains("not valid kebab-case"));
    }

    #[test]
    fn validate_valid_config_passes() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("valid.toml");
        fs::write(
            &path,
            r#"
[[source]]
id = "my-source"
url = "/tmp/skills"
type = "local"

[[agent]]
id = "my-agent"
type = "claude"
path = "/tmp/.claude"
"#,
        )
        .unwrap();
        let config = load_from(&path).unwrap();
        assert_eq!(config.source[0].id, "my-source");
        assert_eq!(config.agent[0].id, "my-agent");
    }
}
