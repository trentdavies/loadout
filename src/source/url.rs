use anyhow::{bail, Result};
use std::path::PathBuf;

/// Parsed components of a GitHub tree URL.
#[derive(Debug, Clone, Default)]
pub struct GitTreeInfo {
    /// Branch or tag extracted from `/tree/<ref>/...`.
    pub tree_ref: Option<String>,
    /// Subpath within the repo from `/tree/<ref>/<path>`.
    pub subpath: Option<String>,
}

/// Resolved source URL — local path, git remote, or archive file.
#[derive(Debug, Clone)]
pub enum SourceUrl {
    /// Local filesystem path (already resolved to absolute).
    Local(PathBuf),
    /// Git repository URL with optional tree info.
    Git(String, GitTreeInfo),
    /// Archive file (.zip or .skill).
    Archive(PathBuf),
}

impl SourceUrl {
    /// Parse a user-provided URL string into a SourceUrl.
    ///
    /// Supports:
    /// - Local paths: `/abs/path`, `./rel/path`, `~/home/path`, `file:///path`
    /// - Git URLs: `https://...`, `git://...`, `ssh://...`, `git@...`
    /// - GitHub shorthand: `github.com/org/repo`, `org/repo`
    /// - GitHub tree URLs: `https://github.com/org/repo/tree/ref/path`
    pub fn parse(input: &str) -> Result<Self> {
        // file:// protocol
        if let Some(path) = input.strip_prefix("file://") {
            let resolved = resolve_path(path)?;
            return Ok(SourceUrl::Local(resolved));
        }

        // Git protocols
        if input.starts_with("git://") || input.starts_with("ssh://") || input.starts_with("git@") {
            return Ok(SourceUrl::Git(input.to_string(), GitTreeInfo::default()));
        }

        // HTTPS — treat .git suffix or github/gitlab hosts as git
        if input.starts_with("https://") || input.starts_with("http://") {
            let (repo_url, tree_info) = parse_github_tree_url(input);
            return Ok(SourceUrl::Git(repo_url, tree_info));
        }

        // GitHub shorthand: github.com/org/repo or github.com/org/repo/tree/ref/path
        if input.starts_with("github.com/") || input.starts_with("gitlab.com/") {
            let full = format!("https://{}", input);
            let (repo_url, tree_info) = parse_github_tree_url(&full);
            return Ok(SourceUrl::Git(repo_url, tree_info));
        }

        // Archive files: .zip or .skill
        if input.ends_with(".zip") || input.ends_with(".skill") {
            let resolved = resolve_path(input)?;
            return Ok(SourceUrl::Archive(resolved));
        }

        // Local path: absolute, relative, or home-relative
        if input.starts_with('/')
            || input.starts_with("./")
            || input.starts_with("../")
            || input.starts_with('~')
            || PathBuf::from(input).exists()
        {
            let resolved = resolve_path(input)?;
            return Ok(SourceUrl::Local(resolved));
        }

        // org/repo shorthand — two segments only
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok(SourceUrl::Git(
                format!("https://github.com/{}.git", input),
                GitTreeInfo::default(),
            ));
        }

        // org/repo/tree/ref/... shorthand — treat as GitHub URL
        if parts.len() >= 4 && parts[2] == "tree" {
            let full = format!("https://github.com/{}", input);
            let (repo_url, tree_info) = parse_github_tree_url(&full);
            return Ok(SourceUrl::Git(repo_url, tree_info));
        }

        bail!("cannot resolve source URL: {}", input);
    }

    /// Derive a default source name from the URL.
    ///
    /// Rules:
    /// - GitHub/GitLab URLs without subpath → org name (e.g. `anthropics` from `github.com/anthropics/skills`)
    /// - `/tree/ref` with no subpath → org name
    /// - `/tree/ref/path/to/dir` → leaf dir name, with leading dot stripped
    /// - Non-GitHub git URLs without org → repo name
    pub fn default_name(&self) -> String {
        match self {
            SourceUrl::Local(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string(),
            SourceUrl::Git(url, tree_info) => {
                if let Some(ref subpath) = tree_info.subpath {
                    // Use leaf dir from subpath, strip leading dot
                    let leaf = subpath.rsplit('/').next().unwrap_or(subpath);
                    leaf.strip_prefix('.').unwrap_or(leaf).to_string()
                } else {
                    let cleaned = url.trim_end_matches(".git");
                    let base = if let Some(pos) = cleaned.find("/tree/") {
                        &cleaned[..pos]
                    } else {
                        cleaned
                    };
                    // For GitHub/GitLab URLs, use org name; otherwise repo name
                    if let Some(org) = extract_git_org(base) {
                        org
                    } else {
                        base.rsplit('/').next().unwrap_or("unnamed").to_string()
                    }
                }
            }
            SourceUrl::Archive(path) => path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string(),
        }
    }

    /// The type string for config serialization.
    pub fn source_type(&self) -> &'static str {
        match self {
            SourceUrl::Local(_) => "local",
            SourceUrl::Git(..) => "git",
            SourceUrl::Archive(_) => "archive",
        }
    }

    /// The URL string for config serialization (preserves the original full URL).
    pub fn url_string(&self) -> String {
        match self {
            SourceUrl::Local(path) => path.display().to_string(),
            SourceUrl::Git(url, _) => url.clone(),
            SourceUrl::Archive(path) => path.display().to_string(),
        }
    }

    /// The clean repo URL for git clone (without /tree/... suffix).
    pub fn clone_url(&self) -> String {
        match self {
            SourceUrl::Git(url, _) => url.clone(),
            other => other.url_string(),
        }
    }

    /// The tree ref extracted from a `/tree/<ref>/...` URL, if any.
    pub fn tree_ref(&self) -> Option<&str> {
        match self {
            SourceUrl::Git(_, info) => info.tree_ref.as_deref(),
            _ => None,
        }
    }

    /// The subpath extracted from a `/tree/<ref>/<path>` URL, if any.
    pub fn subpath(&self) -> Option<&str> {
        match self {
            SourceUrl::Git(_, info) => info.subpath.as_deref(),
            _ => None,
        }
    }
}

/// Extract the org/owner segment from a GitHub/GitLab URL or git@... shorthand.
///
/// Returns `Some(org)` for URLs like:
/// - `https://github.com/org/repo` → `org`
/// - `git@github.com:org/repo` → `org`
///
/// Returns `None` for unrecognized patterns.
fn extract_git_org(url: &str) -> Option<String> {
    // HTTPS: https://github.com/org/repo → split path, org is first segment
    for host in &["github.com/", "gitlab.com/"] {
        if let Some(after) = url.split_once(host).map(|(_, rest)| rest) {
            let segments: Vec<&str> = after.splitn(3, '/').collect();
            if segments.len() >= 2 && !segments[0].is_empty() {
                return Some(segments[0].to_string());
            }
        }
    }

    // git@ shorthand: git@github.com:org/repo
    if url.starts_with("git@") {
        if let Some(after_colon) = url.split_once(':').map(|(_, rest)| rest) {
            let segments: Vec<&str> = after_colon.splitn(2, '/').collect();
            if !segments[0].is_empty() {
                return Some(segments[0].to_string());
            }
        }
    }

    None
}

/// Parse a GitHub/GitLab URL, extracting `/tree/<ref>/<path>` components.
///
/// Returns `(clean_repo_url, GitTreeInfo)`.
/// The repo URL has `/tree/...` stripped and `.git` appended if needed.
fn parse_github_tree_url(url: &str) -> (String, GitTreeInfo) {
    // Look for /tree/ in the URL path
    if let Some(tree_pos) = url.find("/tree/") {
        let repo_part = &url[..tree_pos];
        let after_tree = &url[tree_pos + 6..]; // skip "/tree/"

        // Split into ref and optional subpath at the first /
        let (tree_ref, subpath) = match after_tree.find('/') {
            Some(slash) => {
                let r = &after_tree[..slash];
                let p = &after_tree[slash + 1..];
                let subpath = if p.is_empty() {
                    None
                } else {
                    Some(p.to_string())
                };
                (Some(r.to_string()), subpath)
            }
            None => {
                if after_tree.is_empty() {
                    (None, None)
                } else {
                    (Some(after_tree.to_string()), None)
                }
            }
        };

        (repo_part.to_string(), GitTreeInfo { tree_ref, subpath })
    } else {
        (url.to_string(), GitTreeInfo::default())
    }
}

/// Resolve a path string to an absolute PathBuf.
fn resolve_path(input: &str) -> Result<PathBuf> {
    let expanded = if let Some(rest) = input.strip_prefix("~/") {
        dirs::home_dir()
            .map(|h| h.join(rest))
            .unwrap_or_else(|| PathBuf::from(input))
    } else if input == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from(input))
    } else {
        PathBuf::from(input)
    };

    // Canonicalize if the path exists, otherwise use as-is for absolute paths
    if expanded.exists() {
        Ok(expanded.canonicalize()?)
    } else if expanded.is_absolute() {
        Ok(expanded)
    } else {
        // Relative path — resolve against CWD
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(expanded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_absolute_path() {
        match SourceUrl::parse("/tmp/skills").unwrap() {
            SourceUrl::Local(p) => assert!(p.is_absolute()),
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn parse_file_protocol() {
        match SourceUrl::parse("file:///tmp/skills").unwrap() {
            SourceUrl::Local(p) => assert_eq!(p, PathBuf::from("/tmp/skills")),
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn parse_https_github() {
        match SourceUrl::parse("https://github.com/org/repo.git").unwrap() {
            SourceUrl::Git(url, _) => assert!(url.contains("github.com")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_git_protocol() {
        match SourceUrl::parse("git://example.com/repo.git").unwrap() {
            SourceUrl::Git(url, _) => assert!(url.starts_with("git://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_ssh_protocol() {
        match SourceUrl::parse("ssh://git@example.com/repo.git").unwrap() {
            SourceUrl::Git(url, _) => assert!(url.starts_with("ssh://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_git_at_shorthand() {
        match SourceUrl::parse("git@github.com:org/repo.git").unwrap() {
            SourceUrl::Git(url, _) => assert!(url.starts_with("git@")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_github_shorthand() {
        match SourceUrl::parse("github.com/org/repo").unwrap() {
            SourceUrl::Git(url, _) => {
                assert!(url.starts_with("https://"));
            }
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_org_repo_shorthand() {
        match SourceUrl::parse("myorg/myrepo").unwrap() {
            SourceUrl::Git(url, _) => {
                assert!(url.contains("github.com"));
                assert!(url.contains("myorg/myrepo"));
            }
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn default_name_local() {
        let url = SourceUrl::Local(PathBuf::from("/home/user/my-skills"));
        assert_eq!(url.default_name(), "my-skills");
    }

    #[test]
    fn default_name_git() {
        let url = SourceUrl::Git(
            "https://github.com/org/cool-skills.git".to_string(),
            GitTreeInfo::default(),
        );
        assert_eq!(url.default_name(), "org");
    }

    #[test]
    fn source_type_values() {
        assert_eq!(
            SourceUrl::Local(PathBuf::from("/tmp")).source_type(),
            "local"
        );
        assert_eq!(
            SourceUrl::Git("https://x.com".to_string(), GitTreeInfo::default()).source_type(),
            "git"
        );
    }

    #[test]
    fn parse_relative_dot_slash() {
        match SourceUrl::parse("./some-local-path").unwrap() {
            SourceUrl::Local(p) => assert!(p.is_absolute()),
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn parse_relative_dot_dot() {
        match SourceUrl::parse("../parent-path").unwrap() {
            SourceUrl::Local(p) => assert!(p.is_absolute()),
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn parse_home_expansion() {
        match SourceUrl::parse("~/some/path").unwrap() {
            SourceUrl::Local(p) => {
                assert!(p.is_absolute());
                assert!(!p.to_string_lossy().contains('~'));
            }
            _ => panic!("expected Local"),
        }
    }

    #[test]
    fn parse_http_url_as_git() {
        match SourceUrl::parse("http://example.com/repo.git").unwrap() {
            SourceUrl::Git(url, _) => assert!(url.starts_with("http://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_invalid_input_errors() {
        assert!(SourceUrl::parse("not/a/valid/multi/segment").is_err());
    }

    #[test]
    fn parse_empty_string_errors() {
        let result = SourceUrl::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn url_string_local() {
        let url = SourceUrl::Local(PathBuf::from("/tmp/skills"));
        assert_eq!(url.url_string(), "/tmp/skills");
    }

    #[test]
    fn url_string_git() {
        let url = SourceUrl::Git(
            "https://github.com/org/repo.git".to_string(),
            GitTreeInfo::default(),
        );
        assert_eq!(url.url_string(), "https://github.com/org/repo.git");
    }

    #[test]
    fn parse_zip_file() {
        match SourceUrl::parse("/tmp/my-plugin.zip").unwrap() {
            SourceUrl::Archive(p) => assert_eq!(p, PathBuf::from("/tmp/my-plugin.zip")),
            _ => panic!("expected Archive"),
        }
    }

    #[test]
    fn parse_skill_file() {
        match SourceUrl::parse("/tmp/helper.skill").unwrap() {
            SourceUrl::Archive(p) => assert_eq!(p, PathBuf::from("/tmp/helper.skill")),
            _ => panic!("expected Archive"),
        }
    }

    #[test]
    fn parse_relative_zip() {
        match SourceUrl::parse("./plugins/test.zip").unwrap() {
            SourceUrl::Archive(p) => assert!(p.is_absolute()),
            _ => panic!("expected Archive"),
        }
    }

    #[test]
    fn parse_non_archive_local_not_archive() {
        match SourceUrl::parse("/tmp/my-skill.md").unwrap() {
            SourceUrl::Local(_) => {}
            _ => panic!("expected Local for .md file"),
        }
    }

    #[test]
    fn default_name_archive() {
        let url = SourceUrl::Archive(PathBuf::from("/tmp/cool-plugin.zip"));
        assert_eq!(url.default_name(), "cool-plugin");
    }

    #[test]
    fn default_name_skill_archive() {
        let url = SourceUrl::Archive(PathBuf::from("/tmp/helper.skill"));
        assert_eq!(url.default_name(), "helper");
    }

    #[test]
    fn source_type_archive() {
        assert_eq!(
            SourceUrl::Archive(PathBuf::from("/tmp/x.zip")).source_type(),
            "archive"
        );
    }

    #[test]
    fn url_string_archive() {
        let url = SourceUrl::Archive(PathBuf::from("/tmp/x.zip"));
        assert_eq!(url.url_string(), "/tmp/x.zip");
    }

    // -----------------------------------------------------------------------
    // Tree URL parsing tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_tree_url_extracts_ref_and_subpath() {
        let url = SourceUrl::parse("https://github.com/org/repo/tree/main/some/path").unwrap();
        match &url {
            SourceUrl::Git(repo, info) => {
                assert_eq!(repo, "https://github.com/org/repo");
                assert_eq!(info.tree_ref.as_deref(), Some("main"));
                assert_eq!(info.subpath.as_deref(), Some("some/path"));
            }
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_tree_url_ref_only() {
        let url = SourceUrl::parse("https://github.com/org/repo/tree/main").unwrap();
        match &url {
            SourceUrl::Git(repo, info) => {
                assert_eq!(repo, "https://github.com/org/repo");
                assert_eq!(info.tree_ref.as_deref(), Some("main"));
                assert!(info.subpath.is_none());
            }
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_tree_url_no_tree() {
        let url = SourceUrl::parse("https://github.com/org/repo").unwrap();
        match &url {
            SourceUrl::Git(_, info) => {
                assert!(info.tree_ref.is_none());
                assert!(info.subpath.is_none());
            }
            _ => panic!("expected Git"),
        }
    }

    // -----------------------------------------------------------------------
    // default_name — tree URL tests
    // -----------------------------------------------------------------------

    #[test]
    fn default_name_repo_only() {
        let url = SourceUrl::parse("https://github.com/anthropics/skills").unwrap();
        assert_eq!(url.default_name(), "anthropics");
    }

    #[test]
    fn default_name_tree_ref_only() {
        let url = SourceUrl::parse("https://github.com/anthropics/skills/tree/main").unwrap();
        assert_eq!(url.default_name(), "anthropics");
    }

    #[test]
    fn default_name_tree_with_subpath() {
        let url =
            SourceUrl::parse("https://github.com/anthropics/skills/tree/main/skills/claude-api")
                .unwrap();
        assert_eq!(url.default_name(), "claude-api");
    }

    #[test]
    fn default_name_tree_hidden_dir() {
        let url =
            SourceUrl::parse("https://github.com/openai/skills/tree/main/skills/.curated").unwrap();
        assert_eq!(url.default_name(), "curated");
    }

    #[test]
    fn default_name_tree_subpath_single_dir() {
        let url = SourceUrl::parse(
            "https://github.com/anthropics/knowledge-work-plugins/tree/main/productivity",
        )
        .unwrap();
        assert_eq!(url.default_name(), "productivity");
    }

    #[test]
    fn default_name_non_github_git() {
        let url = SourceUrl::Git(
            "git://example.com/repo.git".to_string(),
            GitTreeInfo::default(),
        );
        // Non-GitHub URL — falls back to repo name
        assert_eq!(url.default_name(), "repo");
    }

    #[test]
    fn default_name_git_at_shorthand() {
        let url = SourceUrl::Git(
            "git@github.com:anthropics/skills.git".to_string(),
            GitTreeInfo::default(),
        );
        assert_eq!(url.default_name(), "anthropics");
    }
}
