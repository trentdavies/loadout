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
        SourceUrl::Archive(path) => fetch_archive(path, cache_dir),
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

/// Extract a zip/skill archive into the cache directory.
fn fetch_archive(archive_path: &Path, cache_dir: &Path) -> Result<PathBuf> {
    if !archive_path.exists() {
        anyhow::bail!("archive not found: {}", archive_path.display());
    }

    fs::create_dir_all(cache_dir)
        .with_context(|| format!("failed to create cache dir: {}", cache_dir.display()))?;

    let file = fs::File::open(archive_path)
        .with_context(|| format!("failed to open archive: {}", archive_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("failed to read archive: {}", archive_path.display()))?;

    const MAX_FILES: usize = 10_000;
    const MAX_SIZE: u64 = 100 * 1024 * 1024; // 100MB

    if archive.len() > MAX_FILES {
        anyhow::bail!("archive exceeds maximum file count (10,000)");
    }

    let mut total_size: u64 = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // Skip directories and paths with suspicious components
        if name.contains("..") {
            continue;
        }

        let out_path = cache_dir.join(&name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            total_size += entry.size();
            if total_size > MAX_SIZE {
                anyhow::bail!("archive exceeds maximum unpacked size (100MB)");
            }
            let mut outfile = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(cache_dir.to_path_buf())
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

    /// Helper to create a zip file in memory and write it to disk.
    fn create_test_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in files {
            zip.start_file(*name, options).unwrap();
            std::io::Write::write_all(&mut zip, content).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn fetch_archive_valid_zip() {
        let tmp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        let zip_path = tmp.path().join("plugin.zip");
        create_test_zip(&zip_path, &[
            ("SKILL.md", b"---\nname: test\ndescription: d\n---\n"),
            ("extra.txt", b"data"),
        ]);

        let url = SourceUrl::Archive(zip_path);
        let cache_dest = cache.path().join("test-archive");
        let result = fetch(&url, &cache_dest).unwrap();
        assert!(result.join("SKILL.md").exists());
        assert!(result.join("extra.txt").exists());
    }

    #[test]
    fn fetch_archive_skill_file() {
        let tmp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        let skill_path = tmp.path().join("helper.skill");
        create_test_zip(&skill_path, &[
            ("SKILL.md", b"---\nname: helper\ndescription: d\n---\n"),
        ]);

        let url = SourceUrl::Archive(skill_path);
        let cache_dest = cache.path().join("test-skill");
        let result = fetch(&url, &cache_dest).unwrap();
        assert!(result.join("SKILL.md").exists());
    }

    #[test]
    fn fetch_archive_not_found() {
        let cache = TempDir::new().unwrap();
        let url = SourceUrl::Archive(PathBuf::from("/nonexistent/archive.zip"));
        let result = fetch(&url, &cache.path().join("out"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("archive not found"));
    }
}
