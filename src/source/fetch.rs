use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::SourceUrl;

/// Fetch a source into the local cache directory.
/// Returns the path to the cached source content.
pub fn fetch(source_url: &SourceUrl, cache_dir: &Path, git_ref: Option<&str>) -> Result<PathBuf> {
    fetch_with_mode(source_url, cache_dir, git_ref, false)
}

/// Fetch a source with an explicit symlink mode.
/// When `symlink` is true and the source is a local directory, create a symlink
/// instead of copying. SingleFile sources always copy regardless of mode.
pub fn fetch_with_mode(
    source_url: &SourceUrl,
    cache_dir: &Path,
    git_ref: Option<&str>,
    symlink: bool,
) -> Result<PathBuf> {
    match source_url {
        SourceUrl::Local(path) => fetch_local(path, cache_dir, symlink),
        SourceUrl::Git(url, _) => fetch_git(url, cache_dir, git_ref),
        SourceUrl::Archive(path) => fetch_archive(path, cache_dir),
    }
}

/// Fetch a local source into the cache. When `symlink` is true and the source
/// is a directory, create a symlink instead of copying. Files always copy.
fn fetch_local(source_path: &Path, cache_dir: &Path, symlink: bool) -> Result<PathBuf> {
    if !source_path.exists() {
        anyhow::bail!("source path does not exist: {}", source_path.display());
    }

    if source_path.is_file() {
        // Single file source — always copy regardless of symlink mode
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("failed to create cache dir: {}", cache_dir.display()))?;
        let file_name = source_path
            .file_name()
            .context("source path has no file name")?;
        let dest = cache_dir.join(file_name);
        fs::copy(source_path, &dest).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source_path.display(),
                dest.display()
            )
        })?;
        return Ok(cache_dir.to_path_buf());
    }

    if symlink {
        return fetch_local_symlink(source_path, cache_dir);
    }

    // Directory source — recursive copy
    copy_dir_recursive(source_path, cache_dir).with_context(|| {
        format!(
            "failed to copy source {} to {}",
            source_path.display(),
            cache_dir.display()
        )
    })?;

    Ok(cache_dir.to_path_buf())
}

/// Create a symlink from cache_dir to source_path.
/// Falls back to copy if symlink creation fails (e.g., cross-device).
fn fetch_local_symlink(source_path: &Path, cache_dir: &Path) -> Result<PathBuf> {
    // Ensure parent of cache_dir exists
    if let Some(parent) = cache_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(source_path, cache_dir) {
            Ok(()) => return Ok(cache_dir.to_path_buf()),
            Err(e) => {
                eprintln!("warning: symlink failed ({}), falling back to copy", e);
            }
        }
    }

    // Fallback: copy
    copy_dir_recursive(source_path, cache_dir).with_context(|| {
        format!(
            "failed to copy source {} to {}",
            source_path.display(),
            cache_dir.display()
        )
    })?;

    Ok(cache_dir.to_path_buf())
}

/// Clone a git repository into the cache directory using the git CLI.
/// Uses the system git so SSH config, agent forwarding, and host aliases all work.
/// When `git_ref` is provided, clones that specific branch or tag.
fn fetch_git(url: &str, cache_dir: &Path, git_ref: Option<&str>) -> Result<PathBuf> {
    if cache_dir.exists() {
        return update_git(cache_dir, git_ref);
    }

    fs::create_dir_all(cache_dir.parent().unwrap_or(cache_dir))?;

    let mut args = vec!["clone", "--depth", "1"];
    let ref_string;
    if let Some(r) = git_ref {
        ref_string = r.to_string();
        args.push("--branch");
        args.push(&ref_string);
    }
    args.push(url);

    let output = std::process::Command::new("git")
        .args(&args)
        .arg(cache_dir)
        .output()
        .context("failed to run git clone (is git installed?)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("failed to clone {}: {}", url, stderr.trim());
    }

    Ok(cache_dir.to_path_buf())
}

/// The type of a git ref — determines update behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefType {
    /// No ref specified — tracks default branch.
    Latest,
    /// Ref is a branch — tracks that branch.
    Tracking(String),
    /// Ref is a tag — pinned, update warns and skips.
    Pinned(String),
}

/// Detect whether a ref is a tag or branch in the given repo.
pub fn detect_ref_type(git_ref: Option<&str>, repo_path: &Path) -> RefType {
    let r = match git_ref {
        None => return RefType::Latest,
        Some(r) => r,
    };

    if is_tag(r, repo_path) {
        RefType::Pinned(r.to_string())
    } else {
        RefType::Tracking(r.to_string())
    }
}

/// Check if a ref is a tag in the given repo.
pub fn is_tag(git_ref: &str, repo_path: &Path) -> bool {
    let output = std::process::Command::new("git")
        .args(["tag", "--list", git_ref])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

/// Update an existing git clone, respecting the ref.
/// - No ref (latest): fetch + reset to origin/HEAD
/// - Branch (tracking): fetch + reset to origin/<branch>
/// - Tag (pinned): warn and skip
///
/// Returns `Ok(None)` when the update was skipped (pinned tag).
pub fn update_git_ref(repo_path: &Path, git_ref: Option<&str>) -> Result<Option<PathBuf>> {
    let ref_type = detect_ref_type(git_ref, repo_path);

    match ref_type {
        RefType::Pinned(_) => {
            Ok(None) // Caller should warn
        }
        RefType::Tracking(branch) => {
            git_fetch(repo_path)?;
            git_reset(repo_path, &format!("origin/{}", branch))?;
            Ok(Some(repo_path.to_path_buf()))
        }
        RefType::Latest => {
            git_fetch(repo_path)?;
            git_reset(repo_path, "origin/HEAD")?;
            Ok(Some(repo_path.to_path_buf()))
        }
    }
}

/// Update an existing git clone by fetching and resetting.
pub fn update_git(repo_path: &Path, git_ref: Option<&str>) -> Result<PathBuf> {
    let reset_target = match git_ref {
        Some(r) => format!("origin/{}", r),
        None => "origin/HEAD".to_string(),
    };

    git_fetch(repo_path)?;
    git_reset(repo_path, &reset_target)?;
    Ok(repo_path.to_path_buf())
}

fn git_fetch(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(repo_path)
        .output()
        .context("failed to run git fetch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "failed to fetch in {}: {}",
            repo_path.display(),
            stderr.trim()
        );
    }
    Ok(())
}

fn git_reset(repo_path: &Path, target: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["reset", "--hard", target])
        .current_dir(repo_path)
        .output()
        .context("failed to run git reset")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "failed to reset in {}: {}",
            repo_path.display(),
            stderr.trim()
        );
    }
    Ok(())
}

/// Switch a git repo to a different ref. Fetches first, then checks out.
pub fn switch_ref(repo_path: &Path, new_ref: &str) -> Result<()> {
    git_fetch(repo_path)?;

    // Try checking out as a remote branch first, fall back to tag/direct ref
    let output = std::process::Command::new("git")
        .args(["checkout", new_ref])
        .current_dir(repo_path)
        .output()
        .context("failed to run git checkout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("failed to checkout '{}': {}", new_ref, stderr.trim());
    }

    Ok(())
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
        fs::write(
            src.path().join("SKILL.md"),
            "---\nname: x\ndescription: d\n---\n",
        )
        .unwrap();
        fs::write(src.path().join("extra.txt"), "data").unwrap();

        let url = SourceUrl::Local(src.path().to_path_buf());
        let cache_dest = cache.path().join("test-src");
        let result = fetch(&url, &cache_dest, None).unwrap();
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
        let result = fetch(&url, &cache_dest, None).unwrap();
        assert!(result.join("SKILL.md").exists());
    }

    #[test]
    fn fetch_local_nonexistent_errors() {
        let cache = TempDir::new().unwrap();
        let url = SourceUrl::Local(PathBuf::from("/nonexistent/path/xyz"));
        let result = fetch(&url, &cache.path().join("out"), None);
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
        create_test_zip(
            &zip_path,
            &[
                ("SKILL.md", b"---\nname: test\ndescription: d\n---\n"),
                ("extra.txt", b"data"),
            ],
        );

        let url = SourceUrl::Archive(zip_path);
        let cache_dest = cache.path().join("test-archive");
        let result = fetch(&url, &cache_dest, None).unwrap();
        assert!(result.join("SKILL.md").exists());
        assert!(result.join("extra.txt").exists());
    }

    #[test]
    fn fetch_archive_skill_file() {
        let tmp = TempDir::new().unwrap();
        let cache = TempDir::new().unwrap();
        let skill_path = tmp.path().join("helper.skill");
        create_test_zip(
            &skill_path,
            &[("SKILL.md", b"---\nname: helper\ndescription: d\n---\n")],
        );

        let url = SourceUrl::Archive(skill_path);
        let cache_dest = cache.path().join("test-skill");
        let result = fetch(&url, &cache_dest, None).unwrap();
        assert!(result.join("SKILL.md").exists());
    }

    #[test]
    fn fetch_archive_not_found() {
        let cache = TempDir::new().unwrap();
        let url = SourceUrl::Archive(PathBuf::from("/nonexistent/archive.zip"));
        let result = fetch(&url, &cache.path().join("out"), None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("archive not found"));
    }
}
