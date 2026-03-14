use anyhow::{bail, Result};
use std::path::PathBuf;

/// Resolved source URL — local path, git remote, or archive file.
#[derive(Debug, Clone)]
pub enum SourceUrl {
    /// Local filesystem path (already resolved to absolute).
    Local(PathBuf),
    /// Git repository URL.
    Git(String),
    /// Archive file (.zip or .skill).
    Archive(PathBuf),
}

impl SourceUrl {
    /// Parse a user-provided URL string into a SourceUrl.
    ///
    /// Supports:
    /// - Local paths: `/abs/path`, `./rel/path`, `~/home/path`, `file:///path`
    /// - Git URLs: `https://...`, `git://...`, `ssh://...`, `git@...`
    /// - GitHub shorthand: `github.com/org/repo`
    pub fn parse(input: &str) -> Result<Self> {
        // file:// protocol
        if let Some(path) = input.strip_prefix("file://") {
            let resolved = resolve_path(path)?;
            return Ok(SourceUrl::Local(resolved));
        }

        // Git protocols
        if input.starts_with("git://") || input.starts_with("ssh://") || input.starts_with("git@") {
            return Ok(SourceUrl::Git(input.to_string()));
        }

        // HTTPS — treat .git suffix or github/gitlab hosts as git
        if input.starts_with("https://") || input.starts_with("http://") {
            if input.ends_with(".git")
                || input.contains("github.com")
                || input.contains("gitlab.com")
            {
                return Ok(SourceUrl::Git(input.to_string()));
            }
            // Could be an HTTP archive, but for now treat as git
            return Ok(SourceUrl::Git(input.to_string()));
        }

        // GitHub shorthand: github.com/org/repo
        if input.starts_with("github.com/") || input.starts_with("gitlab.com/") {
            return Ok(SourceUrl::Git(format!("https://{}.git", input)));
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

        // If it looks like org/repo (no slashes beyond one), treat as GitHub shorthand
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok(SourceUrl::Git(format!("https://github.com/{}.git", input)));
        }

        bail!("cannot resolve source URL: {}", input);
    }

    /// Derive a default source name from the URL.
    pub fn default_name(&self) -> String {
        match self {
            SourceUrl::Local(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string(),
            SourceUrl::Git(url) => {
                // Extract repo name from URL
                let cleaned = url.trim_end_matches(".git");
                cleaned.rsplit('/').next().unwrap_or("unnamed").to_string()
            }
            SourceUrl::Archive(path) => {
                // Filename without extension
                path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unnamed")
                    .to_string()
            }
        }
    }

    /// The type string for config serialization.
    pub fn source_type(&self) -> &'static str {
        match self {
            SourceUrl::Local(_) => "local",
            SourceUrl::Git(_) => "git",
            SourceUrl::Archive(_) => "archive",
        }
    }

    /// The URL string for config serialization.
    pub fn url_string(&self) -> String {
        match self {
            SourceUrl::Local(path) => path.display().to_string(),
            SourceUrl::Git(url) => url.clone(),
            SourceUrl::Archive(path) => path.display().to_string(),
        }
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
            SourceUrl::Git(url) => assert!(url.contains("github.com")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_git_protocol() {
        match SourceUrl::parse("git://example.com/repo.git").unwrap() {
            SourceUrl::Git(url) => assert!(url.starts_with("git://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_ssh_protocol() {
        match SourceUrl::parse("ssh://git@example.com/repo.git").unwrap() {
            SourceUrl::Git(url) => assert!(url.starts_with("ssh://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_git_at_shorthand() {
        match SourceUrl::parse("git@github.com:org/repo.git").unwrap() {
            SourceUrl::Git(url) => assert!(url.starts_with("git@")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_github_shorthand() {
        match SourceUrl::parse("github.com/org/repo").unwrap() {
            SourceUrl::Git(url) => {
                assert!(url.starts_with("https://"));
                assert!(url.ends_with(".git"));
            }
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_org_repo_shorthand() {
        match SourceUrl::parse("myorg/myrepo").unwrap() {
            SourceUrl::Git(url) => {
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
        let url = SourceUrl::Git("https://github.com/org/cool-skills.git".to_string());
        assert_eq!(url.default_name(), "cool-skills");
    }

    #[test]
    fn source_type_values() {
        assert_eq!(
            SourceUrl::Local(PathBuf::from("/tmp")).source_type(),
            "local"
        );
        assert_eq!(
            SourceUrl::Git("https://x.com".to_string()).source_type(),
            "git"
        );
    }

    #[test]
    fn parse_relative_dot_slash() {
        // ./something resolves to CWD/something
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
            SourceUrl::Git(url) => assert!(url.starts_with("http://")),
            _ => panic!("expected Git"),
        }
    }

    #[test]
    fn parse_invalid_input_errors() {
        // Three-segment path that doesn't exist and isn't a recognized format
        assert!(SourceUrl::parse("not/a/valid/multi/segment").is_err());
    }

    #[test]
    fn parse_empty_string_errors() {
        // Empty string should not be a valid source URL
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
        let url = SourceUrl::Git("https://github.com/org/repo.git".to_string());
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
        // .md file should be Local, not Archive
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
    // default_name — tree URL tests (target behavior for subpath support)
    // -----------------------------------------------------------------------

    #[test]
    fn default_name_repo_only() {
        let url = SourceUrl::Git("https://github.com/anthropics/skills".to_string());
        assert_eq!(url.default_name(), "skills");
    }

    #[test]
    fn default_name_tree_ref_only() {
        // /tree/main with no subpath → repo name
        let url =
            SourceUrl::Git("https://github.com/anthropics/skills/tree/main".to_string());
        assert_eq!(url.default_name(), "skills");
    }

    #[test]
    fn default_name_tree_with_subpath() {
        // /tree/main/skills/claude-api → leaf dir name
        let url = SourceUrl::Git(
            "https://github.com/anthropics/skills/tree/main/skills/claude-api".to_string(),
        );
        assert_eq!(url.default_name(), "claude-api");
    }

    #[test]
    fn default_name_tree_hidden_dir() {
        // /tree/main/skills/.curated → strip leading dot
        let url = SourceUrl::Git(
            "https://github.com/openai/skills/tree/main/skills/.curated".to_string(),
        );
        assert_eq!(url.default_name(), "curated");
    }

    #[test]
    fn default_name_tree_subpath_single_dir() {
        // /tree/main/productivity → leaf dir name
        let url = SourceUrl::Git(
            "https://github.com/anthropics/knowledge-work-plugins/tree/main/productivity"
                .to_string(),
        );
        assert_eq!(url.default_name(), "productivity");
    }
}
