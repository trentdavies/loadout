use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a skill directory with valid SKILL.md frontmatter.
fn make_skill_fixture(parent: &Path, name: &str) {
    let skill_dir = parent.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: Test skill {}\n---\n# {}\n",
            name, name, name
        ),
    )
    .unwrap();
}

/// Helper to create a zip file from a list of (path, content) pairs.
fn create_test_zip(zip_path: &Path, files: &[(&str, &[u8])]) {
    let file = fs::File::create(zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    for (name, content) in files {
        zip.start_file(*name, options).unwrap();
        std::io::Write::write_all(&mut zip, content).unwrap();
    }
    zip.finish().unwrap();
}

// ─── Test: source add with .zip file ────────────────────────────────────────

#[test]
fn source_add_zip_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();
    let cache_dir = data_dir.join("sources");
    fs::create_dir_all(&cache_dir).unwrap();

    // Create a zip with a plugin structure
    let zip_path = tmp.path().join("my-plugin.zip");
    create_test_zip(&zip_path, &[
        (".claude-plugin/plugin.json", br#"{"name": "my-plugin", "version": "1.0"}"#),
        ("skill-a/SKILL.md", b"---\nname: skill-a\ndescription: Skill A\n---\n"),
        ("skill-b/SKILL.md", b"---\nname: skill-b\ndescription: Skill B\n---\n"),
    ]);

    // Parse as archive
    let source_url = skittle::source::SourceUrl::parse(zip_path.to_str().unwrap()).unwrap();
    assert_eq!(source_url.source_type(), "archive");
    assert_eq!(source_url.default_name(), "my-plugin");

    // Fetch (unpack)
    let source_cache = cache_dir.join("my-plugin");
    skittle::source::fetch::fetch(&source_url, &source_cache).unwrap();
    assert!(source_cache.join(".claude-plugin/plugin.json").exists());
    assert!(source_cache.join("skill-a/SKILL.md").exists());

    // Detect
    let structure = skittle::source::detect::detect(&source_cache).unwrap();
    assert!(matches!(structure, skittle::source::detect::SourceStructure::SinglePlugin));

    // Normalize
    let registered = skittle::source::normalize::normalize("my-plugin", &source_cache, &structure).unwrap();
    assert_eq!(registered.plugins.len(), 1);
    assert_eq!(registered.plugins[0].name, "my-plugin");
    assert_eq!(registered.plugins[0].skills.len(), 2);
}

// ─── Test: source add with .skill file ──────────────────────────────────────

#[test]
fn source_add_skill_file_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();

    // Create a .skill file (zip) containing an AgentSkill
    let skill_path = tmp.path().join("helper.skill");
    create_test_zip(&skill_path, &[
        ("SKILL.md", b"---\nname: helper\ndescription: A helper skill\n---\n# Helper\n"),
    ]);

    let source_url = skittle::source::SourceUrl::parse(skill_path.to_str().unwrap()).unwrap();
    assert_eq!(source_url.source_type(), "archive");
    assert_eq!(source_url.default_name(), "helper");

    let source_cache = cache_dir.join("helper");
    skittle::source::fetch::fetch(&source_url, &source_cache).unwrap();
    assert!(source_cache.join("SKILL.md").exists());

    let structure = skittle::source::detect::detect(&source_cache).unwrap();
    assert!(matches!(structure, skittle::source::detect::SourceStructure::SingleSkillDir { .. }));
}

// ─── Test: source add with .claude-plugin directory ─────────────────────────

#[test]
fn source_add_claude_plugin_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("my-claude-plugin");
    fs::create_dir_all(&plugin_dir).unwrap();

    // .claude-plugin/plugin.json
    let cp_dir = plugin_dir.join(".claude-plugin");
    fs::create_dir_all(&cp_dir).unwrap();
    fs::write(
        cp_dir.join("plugin.json"),
        r#"{"name": "claude-tool", "version": "2.0", "author": {"name": "trent"}}"#,
    ).unwrap();

    // Skills
    make_skill_fixture(&plugin_dir, "tool-a");

    // Detect
    let structure = skittle::source::detect::detect(&plugin_dir).unwrap();
    assert!(matches!(structure, skittle::source::detect::SourceStructure::SinglePlugin));

    // Normalize — should pick up .claude-plugin metadata
    let registered = skittle::source::normalize::normalize("my-claude-plugin", &plugin_dir, &structure).unwrap();
    assert_eq!(registered.plugins.len(), 1);
    assert_eq!(registered.plugins[0].name, "claude-tool");
    assert_eq!(registered.plugins[0].version.as_deref(), Some("2.0"));
    assert_eq!(registered.plugins[0].skills.len(), 1);
}

// ─── Test: skittle add delegates to source add ──────────────────────────────

#[test]
fn add_shorthand_parses_correctly() {
    use clap::Parser;
    let cli = skittle::cli::Cli::try_parse_from(["skittle", "add", "/tmp/my-src"]).unwrap();
    match cli.command {
        skittle::cli::Command::Add { url, name } => {
            assert_eq!(url, "/tmp/my-src");
            assert!(name.is_none());
        }
        _ => panic!("expected Add command"),
    }
}

// ─── Test: skittle list delegates to skill list ─────────────────────────────

#[test]
fn list_shorthand_parses() {
    use clap::Parser;
    let cli = skittle::cli::Cli::try_parse_from(["skittle", "list"]).unwrap();
    assert!(matches!(cli.command, skittle::cli::Command::List));
}

// ─── Test: skittle init with URL ────────────────────────────────────────────

#[test]
fn init_with_url_parses() {
    use clap::Parser;
    let cli = skittle::cli::Cli::try_parse_from(["skittle", "init", "https://github.com/org/repo"]).unwrap();
    match cli.command {
        skittle::cli::Command::Init { url } => {
            assert_eq!(url.as_deref(), Some("https://github.com/org/repo"));
        }
        _ => panic!("expected Init command"),
    }
}
