use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use super::SourceUrl;

/// Fetch a source into the local cache directory.
/// Returns the path to the cached source content.
pub fn fetch(source_url: &SourceUrl, cache_dir: &Path) -> Result<PathBuf> {
    match source_url {
        SourceUrl::Local(path) => fetch_local(path, cache_dir),
        SourceUrl::Git(url) => fetch_git(url, cache_dir),
    }
}

/// Copy a local directory into the cache.
fn fetch_local(source_path: &Path, cache_dir: &Path) -> Result<PathBuf> {
    if !source_path.exists() {
        anyhow::bail!("source path does not exist: {}", source_path.display());
    }

    if source_path.is_file() {
        // Single file source — copy it into the cache dir
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("failed to create cache dir: {}", cache_dir.display()))?;
        let file_name = source_path.file_name()
            .context("source path has no file name")?;
        let dest = cache_dir.join(file_name);
        fs::copy(source_path, &dest)
            .with_context(|| format!("failed to copy {} to {}", source_path.display(), dest.display()))?;
        return Ok(cache_dir.to_path_buf());
    }

    // Directory source — recursive copy
    copy_dir_recursive(source_path, cache_dir)
        .with_context(|| format!(
            "failed to copy source {} to {}",
            source_path.display(),
            cache_dir.display()
        ))?;

    Ok(cache_dir.to_path_buf())
}

/// Clone a git repository into the cache directory.
fn fetch_git(url: &str, cache_dir: &Path) -> Result<PathBuf> {
    if cache_dir.exists() {
        // Already cloned — pull instead
        return update_git(cache_dir);
    }

    fs::create_dir_all(cache_dir.parent().unwrap_or(cache_dir))?;

    git2::Repository::clone(url, cache_dir)
        .with_context(|| format!("failed to clone {}", url))?;

    Ok(cache_dir.to_path_buf())
}

/// Update an existing git clone by fetching and resetting to origin/HEAD.
pub fn update_git(repo_path: &Path) -> Result<PathBuf> {
    let repo = git2::Repository::open(repo_path)
        .with_context(|| format!("failed to open git repo at {}", repo_path.display()))?;

    // Fetch origin
    let mut remote = repo.find_remote("origin")
        .context("no 'origin' remote found")?;
    remote.fetch(&["HEAD"], None, None)
        .context("failed to fetch from origin")?;

    // Reset to FETCH_HEAD
    let fetch_head = repo.find_reference("FETCH_HEAD")
        .context("no FETCH_HEAD after fetch")?;
    let commit = fetch_head.peel_to_commit()
        .context("FETCH_HEAD does not point to a commit")?;
    repo.reset(commit.as_object(), git2::ResetType::Hard, None)
        .context("failed to reset to fetched HEAD")?;

    Ok(repo_path.to_path_buf())
}

/// Recursively copy a directory, skipping .git directories.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();

        // Skip .git directories
        if name == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&name);

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
