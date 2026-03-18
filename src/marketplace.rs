/// Generate `.claude-plugin/marketplace.json` from plugin directories in the data dir root.
pub fn generate_local_manifest(data_dir: &std::path::Path) -> anyhow::Result<()> {
    /// Directories in the data dir that are infrastructure, not plugins.
    const SKIP_DIRS: &[&str] = &["external"];

    let mut plugins = Vec::new();

    if data_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(data_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.') || SKIP_DIRS.contains(&dir_name.as_str()) {
                continue;
            }

            let has_plugin_marker = entry.path().join(".claude-plugin").is_dir();
            let has_skills = entry.path().join("skills").is_dir();
            if !has_plugin_marker && !has_skills {
                continue;
            }

            let plugin_json = entry.path().join(".claude-plugin/plugin.json");
            let (name, description) = if plugin_json.exists() {
                if let Ok(manifest) = crate::source::manifest::load_plugin_manifest(&plugin_json) {
                    (manifest.name, manifest.description)
                } else {
                    (dir_name.clone(), None)
                }
            } else {
                (dir_name.clone(), None)
            };

            let mut plugin_entry = serde_json::json!({
                "name": name,
                "source": format!("./{}", dir_name),
            });
            if let Some(desc) = description {
                plugin_entry["description"] = serde_json::Value::String(desc);
            }
            plugins.push(plugin_entry);
        }
    }

    let existing_manifest = data_dir.join(".claude-plugin/marketplace.json");
    let marketplace_name = if existing_manifest.exists() {
        crate::source::manifest::load_marketplace(&existing_manifest)
            .map(|manifest| manifest.name)
            .unwrap_or_else(|_| "local".to_string())
    } else {
        "local".to_string()
    };

    let marketplace = serde_json::json!({
        "name": marketplace_name,
        "plugins": plugins,
    });

    let cp_dir = data_dir.join(".claude-plugin");
    std::fs::create_dir_all(&cp_dir)?;
    std::fs::write(
        cp_dir.join("marketplace.json"),
        serde_json::to_string_pretty(&marketplace)?,
    )?;

    Ok(())
}
