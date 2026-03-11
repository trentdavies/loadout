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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fetch_local_directory() {
        let src = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        fs::write(src.path().join("SKILL.md"), "---\nname: x\ndescription: d\n---\n").unwrap();
        fs::write(src.path().join("extra.txt"), "data").unwrap();

        let url = SourceUrl::Local(src.path().to_path_buf());
        let cache_dest = cache.path().join("test-src");
        let result = fetch(&url, &cache_dest).unwrap();
        assert!(result.join("SKILL.md").exists());
        assert!(result.join("extra.txt").exists());
    }

    #[test]
    fn fetch_local_single_file() {
        let src = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        let skill_file = src.path().join("SKILL.md");
        fs::write(&skill_file, "---\nname: x\ndescription: d\n---\n").unwrap();

        let url = SourceUrl::Local(skill_file);
        let cache_dest = cache.path().join("test-src");
        let result = fetch(&url, &cache_dest).unwrap();
        assert!(result.join("SKILL.md").exists());
    }

    #[test]
    fn fetch_local_nonexistent_errors() {
        let cache = TempDir::new().unwrap();
        let url = SourceUrl::Local(PathBuf::from("/nonexistent/path/xyz"));
        let result = fetch(&url, &cache.path().join("out"));
        assert!(result.is_err());
    }

    #[test]
    fn copy_dir_recursive_skips_git() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        fs::write(src.path().join("file.txt"), "content").unwrap();
        let git_dir = src.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref").unwrap();

        let dest = dst.path().join("out");
        copy_dir_recursive(src.path(), &dest).unwrap();

        assert!(dest.join("file.txt").exists());
        assert!(!dest.join(".git").exists());
    }

    #[test]
    fn copy_dir_recursive_nested() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        let sub = src.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "data").unwrap();

        let dest = dst.path().join("out");
        copy_dir_recursive(src.path(), &dest).unwrap();

        assert!(dest.join("sub").join("nested.txt").exists());
    }
}
