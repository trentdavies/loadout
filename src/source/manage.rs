use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::{Config, SourceConfig, SourceResidence};
use crate::registry::{RegisteredSource, Registry};

use super::{normalize, ParsedSource, SourceUrl};

pub struct PreparedSource {
    pub config: SourceConfig,
    pub registered: RegisteredSource,
}

pub enum RefreshSource {
    Updated(Box<PreparedSource>),
    SkippedPinned { pinned_ref: String },
}

pub struct LocalImport {
    pub source_name: String,
    pub display_name: Option<String>,
    pub plugins: Vec<crate::registry::RegisteredPlugin>,
}

pub fn default_source_residence() -> SourceResidence {
    SourceResidence::External
}

pub fn source_kind_residence(kind: super::SourceKind) -> SourceResidence {
    match kind {
        super::SourceKind::Marketplace => SourceResidence::External,
        super::SourceKind::SinglePlugin
        | super::SourceKind::FlatSkills
        | super::SourceKind::SingleSkillDir
        | super::SourceKind::SingleFile => SourceResidence::Local,
    }
}

pub fn source_storage_root(data_dir: &Path, residence: SourceResidence) -> PathBuf {
    match residence {
        SourceResidence::External => data_dir.join("external"),
        SourceResidence::Local => data_dir.to_path_buf(),
    }
}

pub fn source_storage_path_in(
    data_dir: &Path,
    source_name: &str,
    residence: SourceResidence,
) -> PathBuf {
    source_storage_root(data_dir, residence).join(source_name)
}

pub fn source_storage_path(source_name: &str, residence: SourceResidence) -> PathBuf {
    source_storage_path_in(&crate::config::data_dir(), source_name, residence)
}

pub fn source_storage_path_for_config(source: &SourceConfig) -> PathBuf {
    source_storage_path(&source.name, source.residence)
}

pub fn detect_path(source_url: &SourceUrl, cache_path: &Path) -> PathBuf {
    if let Some(subpath) = source_url.subpath() {
        cache_path.join(subpath)
    } else {
        cache_path.to_path_buf()
    }
}

pub fn build_source_config(
    source_name: &str,
    source_url: &SourceUrl,
    git_ref: Option<String>,
    mode: Option<String>,
    residence: SourceResidence,
) -> SourceConfig {
    SourceConfig {
        name: source_name.to_string(),
        url: source_url.url_string(),
        source_type: source_url.source_type().to_string(),
        r#ref: git_ref,
        mode,
        residence,
    }
}

pub fn prepare_source(
    source_name: &str,
    source_url: &SourceUrl,
    cache_path: &Path,
    git_ref: Option<String>,
    mode: Option<String>,
    residence: SourceResidence,
    overrides: &normalize::Overrides<'_>,
) -> Result<PreparedSource> {
    let parsed = ParsedSource::parse(&detect_path(source_url, cache_path))?
        .with_source_name(source_name)
        .with_url(source_url.url_string());
    let mut registered = normalize::normalize_with(&parsed, overrides)?;
    registered.residence = residence;
    let config = build_source_config(source_name, source_url, git_ref, mode, residence);
    Ok(PreparedSource { config, registered })
}

pub fn import_into_local_source(
    parsed: &ParsedSource,
    overrides: &normalize::Overrides<'_>,
    data_dir: &Path,
) -> Result<LocalImport> {
    if source_kind_residence(parsed.kind) != SourceResidence::Local {
        anyhow::bail!(
            "source kind {:?} cannot be imported into the local source",
            parsed.kind
        );
    }

    let imported = normalize::normalize_with(&parsed.clone().with_source_name("local"), overrides)?;
    let imported_plugin_names: std::collections::BTreeSet<String> = imported
        .plugins
        .iter()
        .map(|plugin| plugin.name.clone())
        .collect();

    match parsed.kind {
        super::SourceKind::SinglePlugin => {
            install_plugin_dir(parsed, &imported.plugins[0], data_dir)?
        }
        super::SourceKind::FlatSkills => {
            install_flat_skills_plugin(&imported.plugins[0], data_dir)?
        }
        super::SourceKind::SingleSkillDir => {
            install_single_skill_dir(parsed, &imported.plugins[0], data_dir)?
        }
        super::SourceKind::SingleFile => {
            install_single_skill_file(parsed, &imported.plugins[0], data_dir)?
        }
        super::SourceKind::Marketplace => unreachable!(),
    }

    crate::marketplace::generate_local_manifest(data_dir)?;

    let local = normalize::normalize(
        &ParsedSource::parse(data_dir)?
            .with_source_name("local")
            .with_url(""),
    )?;
    let plugins = local
        .plugins
        .into_iter()
        .filter(|plugin| imported_plugin_names.contains(&plugin.name))
        .collect();

    Ok(LocalImport {
        source_name: "local".to_string(),
        display_name: local.display_name,
        plugins,
    })
}

pub fn persist_prepared_source(
    config: &mut Config,
    registry: &mut Registry,
    prepared: PreparedSource,
) {
    registry
        .sources
        .retain(|source| source.name != prepared.registered.name);
    registry.sources.push(prepared.registered);

    config
        .source
        .retain(|source| source.name != prepared.config.name);
    config.source.push(prepared.config);
}

fn install_plugin_dir(
    parsed: &ParsedSource,
    plugin: &crate::registry::RegisteredPlugin,
    data_dir: &Path,
) -> Result<()> {
    let target = data_dir.join(&plugin.name);
    if target.exists() {
        anyhow::bail!("local plugin '{}' already exists", plugin.name);
    }

    copy_dir_all(&parsed.path, &target)?;
    ensure_plugin_manifest(&target, plugin)?;
    Ok(())
}

fn install_flat_skills_plugin(
    plugin: &crate::registry::RegisteredPlugin,
    data_dir: &Path,
) -> Result<()> {
    let target = data_dir.join(&plugin.name);
    std::fs::create_dir_all(target.join("skills"))?;

    for skill in &plugin.skills {
        let skill_target = target.join("skills").join(&skill.name);
        if skill_target.exists() {
            anyhow::bail!(
                "local skill '{}' already exists in plugin '{}'",
                skill.name,
                plugin.name
            );
        }
    }

    for skill in &plugin.skills {
        copy_dir_all(&skill.path, &target.join("skills").join(&skill.name))?;
    }

    ensure_plugin_manifest(&target, plugin)?;
    Ok(())
}

fn install_single_skill_dir(
    parsed: &ParsedSource,
    plugin: &crate::registry::RegisteredPlugin,
    data_dir: &Path,
) -> Result<()> {
    let target = data_dir.join(&plugin.name);
    let skill = &plugin.skills[0];
    let skill_target = target.join("skills").join(&skill.name);
    if skill_target.exists() {
        anyhow::bail!(
            "local skill '{}' already exists in plugin '{}'",
            skill.name,
            plugin.name
        );
    }

    std::fs::create_dir_all(target.join("skills"))?;
    copy_dir_all(&parsed.path, &skill_target)?;
    ensure_plugin_manifest(&target, plugin)?;
    Ok(())
}

fn install_single_skill_file(
    parsed: &ParsedSource,
    plugin: &crate::registry::RegisteredPlugin,
    data_dir: &Path,
) -> Result<()> {
    let target = data_dir.join(&plugin.name);
    let skill = &plugin.skills[0];
    let skill_target = target.join("skills").join(&skill.name);
    if skill_target.exists() {
        anyhow::bail!(
            "local skill '{}' already exists in plugin '{}'",
            skill.name,
            plugin.name
        );
    }

    std::fs::create_dir_all(&skill_target)?;
    std::fs::copy(&parsed.path, skill_target.join("SKILL.md")).with_context(|| {
        format!(
            "failed to copy {} into {}",
            parsed.path.display(),
            skill_target.display()
        )
    })?;
    ensure_plugin_manifest(&target, plugin)?;
    Ok(())
}

fn ensure_plugin_manifest(target: &Path, plugin: &crate::registry::RegisteredPlugin) -> Result<()> {
    let claude_plugin_dir = target.join(".claude-plugin");
    std::fs::create_dir_all(&claude_plugin_dir)?;
    let manifest_path = claude_plugin_dir.join("plugin.json");

    let mut manifest = if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&content)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?
    } else {
        serde_json::json!({})
    };

    manifest["name"] = serde_json::Value::String(plugin.name.clone());
    if let Some(version) = &plugin.version {
        manifest["version"] = serde_json::Value::String(version.clone());
    }
    if let Some(description) = &plugin.description {
        manifest["description"] = serde_json::Value::String(description.clone());
    }

    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

pub fn refresh_source(
    source: &SourceConfig,
    cache_path: &Path,
    update_ref: Option<&str>,
) -> Result<RefreshSource> {
    let source_url = SourceUrl::parse(&source.url)?;

    match &source_url {
        SourceUrl::Local(_) => {
            if source.mode.as_deref() != Some("symlink") {
                if cache_path.exists() {
                    std::fs::remove_dir_all(cache_path)?;
                }
                super::fetch::fetch(&source_url, cache_path, None)?;
            }
        }
        SourceUrl::Git(..) => {
            if let Some(new_ref) = update_ref {
                if !cache_path.exists() {
                    let effective_ref = if new_ref == "latest" {
                        None
                    } else {
                        Some(new_ref)
                    };
                    super::fetch::fetch(&source_url, cache_path, effective_ref)?;
                } else if new_ref == "latest" {
                    super::fetch::update_git(cache_path, None)?;
                } else {
                    super::fetch::switch_ref(cache_path, new_ref)?;
                }
            } else if cache_path.exists() {
                match super::fetch::update_git_ref(cache_path, source.r#ref.as_deref())? {
                    Some(_) => {}
                    None => {
                        return Ok(RefreshSource::SkippedPinned {
                            pinned_ref: source
                                .r#ref
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                        });
                    }
                }
            } else {
                super::fetch::fetch(&source_url, cache_path, source.r#ref.as_deref())?;
            }
        }
        SourceUrl::Archive(_) => {
            if cache_path.exists() {
                std::fs::remove_dir_all(cache_path)?;
            }
            super::fetch::fetch(&source_url, cache_path, None)?;
        }
    }

    let git_ref = if let Some(new_ref) = update_ref {
        if new_ref == "latest" {
            None
        } else {
            Some(new_ref.to_string())
        }
    } else {
        source.r#ref.clone()
    };

    Ok(RefreshSource::Updated(Box::new(prepare_source(
        &source.name,
        &source_url,
        cache_path,
        git_ref,
        source.mode.clone(),
        source.residence,
        &normalize::Overrides::default(),
    )?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn import_flat_skills_from_repo_root_uses_nested_skills_only() {
        let tmp = TempDir::new().unwrap();
        let repo_root = tmp.path().join("repo");
        let data_dir = tmp.path().join("data");

        fs::create_dir_all(repo_root.join("skills").join("pptx").join("templates")).unwrap();
        fs::write(repo_root.join("README.md"), "# docs").unwrap();
        fs::write(
            repo_root.join("skills").join("pptx").join("SKILL.md"),
            "---\nname: pptx\ndescription: PowerPoint skill\n---\nbody",
        )
        .unwrap();
        fs::write(
            repo_root
                .join("skills")
                .join("pptx")
                .join("templates")
                .join("template.pptx"),
            "template",
        )
        .unwrap();

        let parsed = ParsedSource::parse(&repo_root)
            .unwrap()
            .with_source_name("slides");
        assert_eq!(parsed.kind, super::super::parsed::SourceKind::FlatSkills);

        let imported = import_into_local_source(
            &parsed,
            &normalize::Overrides {
                plugin: Some("slides"),
                skill: None,
            },
            &data_dir,
        )
        .unwrap();

        assert_eq!(imported.plugins.len(), 1);
        assert_eq!(imported.plugins[0].name, "slides");
        assert_eq!(imported.plugins[0].skills.len(), 1);
        assert!(data_dir
            .join("slides")
            .join("skills")
            .join("pptx")
            .join("SKILL.md")
            .exists());
        assert!(data_dir
            .join("slides")
            .join("skills")
            .join("pptx")
            .join("templates")
            .join("template.pptx")
            .exists());
        assert!(!data_dir.join("slides").join("README.md").exists());
    }
}
