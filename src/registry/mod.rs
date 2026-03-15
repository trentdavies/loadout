pub mod types;

pub use types::*;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Load the registry index from disk.
/// Registry lives at `<data_dir>/.loadout/registry.json`.
pub fn load_registry(data_dir: &Path) -> Result<Registry> {
    let internal = data_dir.join(".loadout");
    let path = internal.join("registry.json");
    // Also check legacy location for migration
    let legacy_path = data_dir.join("registry.json");
    let read_path = if path.exists() {
        path
    } else if legacy_path.exists() {
        legacy_path
    } else {
        return Ok(Registry::default());
    };
    let content = fs::read_to_string(&read_path)
        .with_context(|| format!("failed to read {}", read_path.display()))?;
    let registry: Registry = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", read_path.display()))?;
    Ok(registry)
}

/// Save the registry index to disk.
/// Registry lives at `<data_dir>/.loadout/registry.json`.
pub fn save_registry(registry: &Registry, data_dir: &Path) -> Result<()> {
    let internal = data_dir.join(".loadout");
    fs::create_dir_all(&internal)?;
    let path = internal.join("registry.json");
    let content = serde_json::to_string_pretty(registry).context("failed to serialize registry")?;
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

impl Registry {
    /// Find a plugin by name across all sources.
    /// Returns (source_name, plugin) tuple.
    pub fn find_plugin(&self, name: &str) -> Option<(&str, &RegisteredPlugin)> {
        for src in &self.sources {
            for plugin in &src.plugins {
                if plugin.name == name {
                    return Some((&src.name, plugin));
                }
            }
        }
        None
    }

    /// Find a skill by identity string.
    /// Accepts `plugin/skill` or `source:plugin/skill`.
    /// Returns (source_name, plugin_name, skill) tuple.
    pub fn find_skill(&self, identity: &str) -> Result<(&str, &str, &RegisteredSkill)> {
        let (source_filter, plugin_name, skill_name) = parse_skill_identity(identity)?;

        let mut matches = Vec::new();

        for src in &self.sources {
            if let Some(ref filter) = source_filter {
                if src.name != *filter {
                    continue;
                }
            }
            for plugin in &src.plugins {
                if plugin.name != plugin_name {
                    continue;
                }
                for skill in &plugin.skills {
                    if skill.name == skill_name {
                        matches.push((&*src.name, &*plugin.name, skill));
                    }
                }
            }
        }

        match matches.len() {
            0 => anyhow::bail!("skill '{}' not found", identity),
            1 => Ok((matches[0].0, matches[0].1, matches[0].2)),
            _ => {
                let sources: Vec<String> = matches
                    .iter()
                    .map(|(s, p, sk)| format!("  {}:{}/{}", s, p, sk.name))
                    .collect();
                anyhow::bail!(
                    "skill '{}' is ambiguous. Use fully qualified form:\n{}",
                    identity,
                    sources.join("\n")
                );
            }
        }
    }

    /// Iterate all (source_name, plugin, skill) triples.
    pub fn all_skills(&self) -> Vec<(&str, &RegisteredPlugin, &RegisteredSkill)> {
        let mut result = Vec::new();
        for src in &self.sources {
            for plugin in &src.plugins {
                for skill in &plugin.skills {
                    result.push((src.name.as_str(), plugin, skill));
                }
            }
        }
        result
    }

    /// Match skills whose full identity (`source:plugin/skill`) matches a glob pattern.
    ///
    /// Pattern forms:
    /// - `source:plugin/skill` — fully qualified, matched against full identity
    /// - `plugin/skill` — matched against full identity with any source
    /// - `keyword*` (no `:` or `/`) — freeform, matched against each component
    ///   (source, plugin, skill name) individually
    pub fn match_skills(&self, pattern: &str) -> Vec<(&str, &RegisteredPlugin, &RegisteredSkill)> {
        let freeform = !pattern.contains(':') && !pattern.contains('/');
        let expanded = expand_pattern(pattern);
        let mut result = Vec::new();
        for src in &self.sources {
            for plugin in &src.plugins {
                for skill in &plugin.skills {
                    let matched = if freeform {
                        let flat = format!("{}-{}-{}", src.name, plugin.name, skill.name);
                        glob_match::glob_match(&expanded, &flat)
                            || glob_match::glob_match(&expanded, &src.name)
                            || glob_match::glob_match(&expanded, &plugin.name)
                            || glob_match::glob_match(&expanded, &skill.name)
                    } else {
                        let identity = format!("{}:{}/{}", src.name, plugin.name, skill.name);
                        glob_match::glob_match(&expanded, &identity)
                    };
                    if matched {
                        result.push((src.name.as_str(), plugin, skill));
                    }
                }
            }
        }
        result
    }
}

/// Returns true if the input contains glob metacharacters (`*`, `?`, or `[`).
pub fn is_glob(input: &str) -> bool {
    input.contains('*') || input.contains('?') || input.contains('[')
}

/// Expands a short-form pattern for matching against full identity strings
/// (`source:plugin/skill`).
///
/// - Has `:` → already fully qualified, use as-is
/// - Has `/` → plugin/skill form, prepend `*:` for any source
/// - Otherwise → freeform search, wrap as `*<pattern>*` to match anywhere
///   in the full identity
pub fn expand_pattern(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else if input.contains('/') {
        format!("*:{}", input)
    } else {
        let mut result = String::new();
        if !input.starts_with('*') {
            result.push('*');
        }
        result.push_str(input);
        if !input.ends_with('*') {
            result.push('*');
        }
        result
    }
}

/// Parse a skill identity string into (optional_source, plugin, skill).
fn parse_skill_identity(identity: &str) -> Result<(Option<String>, String, String)> {
    // source:plugin/skill
    if let Some((source, rest)) = identity.split_once(':') {
        if let Some((plugin, skill)) = rest.split_once('/') {
            return Ok((
                Some(source.to_string()),
                plugin.to_string(),
                skill.to_string(),
            ));
        }
        anyhow::bail!(
            "invalid skill identity '{}': expected source:plugin/skill",
            identity
        );
    }

    // plugin/skill
    if let Some((plugin, skill)) = identity.split_once('/') {
        return Ok((None, plugin.to_string(), skill.to_string()));
    }

    anyhow::bail!(
        "invalid skill identity '{}': expected plugin/skill or source:plugin/skill",
        identity
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identity_short_form() {
        let (src, plugin, skill) = parse_skill_identity("myplugin/myskill").unwrap();
        assert!(src.is_none());
        assert_eq!(plugin, "myplugin");
        assert_eq!(skill, "myskill");
    }

    #[test]
    fn parse_identity_full_form() {
        let (src, plugin, skill) = parse_skill_identity("src:plug/sk").unwrap();
        assert_eq!(src, Some("src".to_string()));
        assert_eq!(plugin, "plug");
        assert_eq!(skill, "sk");
    }

    #[test]
    fn parse_identity_no_slash_fails() {
        assert!(parse_skill_identity("justaskill").is_err());
    }

    #[test]
    fn parse_identity_colon_no_slash_fails() {
        assert!(parse_skill_identity("src:noslash").is_err());
    }

    #[test]
    fn find_skill_ambiguous() {
        let mut registry = Registry::default();
        let skill = RegisteredSkill {
            name: "dup".to_string(),
            description: None,
            author: None,
            version: None,
            path: std::path::PathBuf::from("/tmp"),
        };
        for src_name in &["a", "b"] {
            registry.sources.push(RegisteredSource {
                name: src_name.to_string(),
                plugins: vec![RegisteredPlugin {
                    name: "shared".to_string(),
                    version: None,
                    description: None,
                    skills: vec![skill.clone()],
                    path: std::path::PathBuf::from("/tmp"),
                }],
                cache_path: std::path::PathBuf::from("/tmp"),
            });
        }
        let err = registry.find_skill("shared/dup").unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
    }

    #[test]
    fn all_skills_iterates_everything() {
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "s".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "p".to_string(),
                version: None,
                description: None,
                skills: vec![
                    RegisteredSkill {
                        name: "a".to_string(),
                        description: None,
                        author: None,
                        version: None,
                        path: std::path::PathBuf::from("/tmp"),
                    },
                    RegisteredSkill {
                        name: "b".to_string(),
                        description: None,
                        author: None,
                        version: None,
                        path: std::path::PathBuf::from("/tmp"),
                    },
                ],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        assert_eq!(registry.all_skills().len(), 2);
    }

    #[test]
    fn find_plugin_found() {
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "src".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "my-plugin".to_string(),
                version: None,
                description: Some("a plugin".to_string()),
                skills: vec![],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        let (src_name, plugin) = registry.find_plugin("my-plugin").unwrap();
        assert_eq!(src_name, "src");
        assert_eq!(plugin.name, "my-plugin");
    }

    #[test]
    fn find_plugin_not_found() {
        let registry = Registry::default();
        assert!(registry.find_plugin("nonexistent").is_none());
    }

    #[test]
    fn load_registry_corrupted_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("registry.json"), "{broken json").unwrap();
        assert!(load_registry(tmp.path()).is_err());
    }

    #[test]
    fn save_load_registry_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "s".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "p".to_string(),
                version: Some("1.0".to_string()),
                description: None,
                skills: vec![RegisteredSkill {
                    name: "sk".to_string(),
                    description: Some("desc".to_string()),
                    author: None,
                    version: None,
                    path: std::path::PathBuf::from("/tmp/sk"),
                }],
                path: std::path::PathBuf::from("/tmp/p"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        save_registry(&registry, tmp.path()).unwrap();
        let loaded = load_registry(tmp.path()).unwrap();
        assert_eq!(loaded.sources.len(), 1);
        assert_eq!(loaded.sources[0].plugins[0].skills[0].name, "sk");
    }

    #[test]
    fn load_registry_missing_file_returns_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let registry = load_registry(tmp.path()).unwrap();
        assert!(registry.sources.is_empty());
    }

    #[test]
    fn find_skill_not_found() {
        let registry = Registry::default();
        assert!(registry.find_skill("plugin/skill").is_err());
    }

    #[test]
    fn parse_identity_leading_slash() {
        // "/skill" splits to ("", "skill") — empty plugin name, valid parse but won't match anything
        let result = parse_skill_identity("/skill");
        match result {
            Ok((_, plugin, skill)) => {
                assert!(plugin.is_empty());
                assert_eq!(skill, "skill");
            }
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn parse_identity_empty_skill_part() {
        // "plugin/" — slash present but empty skill part
        let result = parse_skill_identity("plugin/");
        // This should parse but produce empty skill name
        match result {
            Ok((_, plugin, skill)) => {
                assert_eq!(plugin, "plugin");
                assert!(skill.is_empty());
            }
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn is_glob_detects_metacharacters() {
        assert!(is_glob("legal/*"));
        assert!(is_glob("*:legal/*"));
        assert!(is_glob("legal/code-?"));
        assert!(is_glob("legal/[abc]"));
        assert!(!is_glob("legal/contract-review"));
        assert!(!is_glob("src:legal/contract-review"));
    }

    #[test]
    fn expand_pattern_prepends_star_colon_for_slash() {
        assert_eq!(expand_pattern("legal/*"), "*:legal/*");
        assert_eq!(expand_pattern("*/code-*"), "*:*/code-*");
    }

    #[test]
    fn expand_pattern_preserves_qualified() {
        assert_eq!(expand_pattern("src:legal/*"), "src:legal/*");
        assert_eq!(expand_pattern("*:*/*"), "*:*/*");
    }

    #[test]
    fn expand_pattern_freeform_wraps_with_stars() {
        assert_eq!(expand_pattern("anthr*"), "*anthr*");
        assert_eq!(expand_pattern("*finan*"), "*finan*");
        assert_eq!(expand_pattern("compl?"), "*compl?*");
        assert_eq!(expand_pattern("legal"), "*legal*");
    }

    fn test_registry() -> Registry {
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "alpha".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "legal".to_string(),
                version: None,
                description: None,
                skills: vec![
                    RegisteredSkill {
                        name: "contract-review".to_string(),
                        description: None,
                        author: None,
                        version: None,
                        path: std::path::PathBuf::from("/tmp"),
                    },
                    RegisteredSkill {
                        name: "compliance".to_string(),
                        description: None,
                        author: None,
                        version: None,
                        path: std::path::PathBuf::from("/tmp"),
                    },
                ],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        registry.sources.push(RegisteredSource {
            name: "beta".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "sales".to_string(),
                version: None,
                description: None,
                skills: vec![RegisteredSkill {
                    name: "call-prep".to_string(),
                    description: None,
                    author: None,
                    version: None,
                    path: std::path::PathBuf::from("/tmp"),
                }],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        registry
    }

    #[test]
    fn match_skills_by_source() {
        let reg = test_registry();
        let matches = reg.match_skills("alpha:*/*");
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|(s, _, _)| *s == "alpha"));
    }

    #[test]
    fn match_skills_by_plugin_short_form() {
        let reg = test_registry();
        let matches = reg.match_skills("legal/*");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn match_skills_by_name_prefix() {
        let reg = test_registry();
        let matches = reg.match_skills("*/c*");
        // contract-review, compliance, call-prep
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn match_skills_no_match() {
        let reg = test_registry();
        let matches = reg.match_skills("nonexistent/*");
        assert!(matches.is_empty());
    }

    #[test]
    fn match_skills_exact_qualified() {
        let reg = test_registry();
        let matches = reg.match_skills("alpha:legal/compliance");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].2.name, "compliance");
    }

    #[test]
    fn match_skills_freeform_source_prefix() {
        // "alph*" should match all skills from source "alpha"
        let reg = test_registry();
        let matches = reg.match_skills("alph*");
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|(s, _, _)| *s == "alpha"));
    }

    #[test]
    fn match_skills_freeform_substring() {
        // "*egal*" should match skills under plugin "legal"
        let reg = test_registry();
        let matches = reg.match_skills("*egal*");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn match_skills_freeform_skill_name() {
        // "call*" should match "call-prep" from beta:sales
        let reg = test_registry();
        let matches = reg.match_skills("call*");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].2.name, "call-prep");
    }

    #[test]
    fn match_skills_freeform_no_match() {
        let reg = test_registry();
        let matches = reg.match_skills("zzz*");
        assert!(matches.is_empty());
    }

    #[test]
    fn match_skills_freeform_contains() {
        // A bare word without glob chars should match as a contains search
        let reg = test_registry();
        // "compl" is not a glob, not a plugin/skill identity — should match "compliance"
        let matches = reg.match_skills("compl");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].2.name, "compliance");
    }

    #[test]
    fn match_skills_freeform_contains_multi() {
        let reg = test_registry();
        // "al" matches source "alpha", plugin "legal", and skill "call-prep"
        let matches = reg.match_skills("al");
        assert_eq!(matches.len(), 3); // 2 from alpha source + call-prep
    }

    #[test]
    fn match_skills_freeform_cross_component() {
        // Pattern spanning multiple identity components should match
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "claude-plugins".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "agent-skills".to_string(),
                version: None,
                description: None,
                skills: vec![RegisteredSkill {
                    name: "my-foo-skill".to_string(),
                    description: None,
                    author: None,
                    version: None,
                    path: std::path::PathBuf::from("/tmp"),
                }],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        let matches = registry.match_skills("cl*sk*");
        assert_eq!(matches.len(), 1, "cl*sk* should match claude-plugins:agent-skills/my-foo-skill");
        assert_eq!(matches[0].2.name, "my-foo-skill");
    }

    #[test]
    fn find_skill_success() {
        let mut registry = Registry::default();
        registry.sources.push(RegisteredSource {
            name: "s".to_string(),
            plugins: vec![RegisteredPlugin {
                name: "p".to_string(),
                version: None,
                description: None,
                skills: vec![RegisteredSkill {
                    name: "sk".to_string(),
                    description: None,
                    author: None,
                    version: None,
                    path: std::path::PathBuf::from("/tmp"),
                }],
                path: std::path::PathBuf::from("/tmp"),
            }],
            cache_path: std::path::PathBuf::from("/tmp"),
        });
        let (src, plug, skill) = registry.find_skill("p/sk").unwrap();
        assert_eq!(src, "s");
        assert_eq!(plug, "p");
        assert_eq!(skill.name, "sk");
    }
}
