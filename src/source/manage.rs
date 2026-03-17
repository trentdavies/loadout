use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::{Config, SourceConfig};
use crate::registry::{RegisteredSource, Registry};

use super::{normalize, ParsedSource, SourceUrl};

pub struct PreparedSource {
    pub config: SourceConfig,
    pub registered: RegisteredSource,
}

pub enum RefreshSource {
    Updated(PreparedSource),
    SkippedPinned { pinned_ref: String },
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
) -> SourceConfig {
    SourceConfig {
        name: source_name.to_string(),
        url: source_url.url_string(),
        source_type: source_url.source_type().to_string(),
        r#ref: git_ref,
        mode,
    }
}

pub fn prepare_source(
    source_name: &str,
    source_url: &SourceUrl,
    cache_path: &Path,
    git_ref: Option<String>,
    mode: Option<String>,
    overrides: &normalize::Overrides<'_>,
) -> Result<PreparedSource> {
    let parsed = ParsedSource::parse(&detect_path(source_url, cache_path))?
        .with_source_name(source_name)
        .with_url(source_url.url_string());
    let registered = normalize::normalize_with(&parsed, overrides)?;
    let config = build_source_config(source_name, source_url, git_ref, mode);
    Ok(PreparedSource { config, registered })
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

    Ok(RefreshSource::Updated(prepare_source(
        &source.name,
        &source_url,
        cache_path,
        git_ref,
        source.mode.clone(),
        &normalize::Overrides::default(),
    )?))
}
