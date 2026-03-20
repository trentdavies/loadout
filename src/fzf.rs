use anyhow::{bail, Result};
use std::io::{IsTerminal, Read, Write};

/// A single item to present in fzf.
/// `display` is shown to the user; `data` is an opaque payload returned on selection.
#[derive(Clone, Debug)]
pub struct FzfItem {
    /// Visible label in fzf (the identity string).
    pub display: String,
    /// Hidden data attached after a tab delimiter (e.g. file path for preview).
    pub data: Option<String>,
}

/// Options controlling fzf behavior.
pub struct FzfOptions {
    /// Allow multi-select (--multi).
    pub multi: bool,
    /// Header text shown at the top of the fzf window.
    pub header: Option<String>,
    /// Preview command (fzf --preview).
    pub preview: Option<String>,
    /// Preview window spec (fzf --preview-window).
    pub preview_window: Option<String>,
    /// Prompt text (fzf --prompt).
    pub prompt: Option<String>,
    /// Extra --bind key:action pairs.
    pub bind: Vec<String>,
}

impl Default for FzfOptions {
    fn default() -> Self {
        Self {
            multi: false,
            header: None,
            preview: None,
            preview_window: None,
            prompt: None,
            bind: Vec::new(),
        }
    }
}

/// Check whether `fzf` is available on PATH.
pub fn fzf_available() -> bool {
    std::process::Command::new("fzf")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run fzf with the given items and options. Returns the selected items.
///
/// If the user cancels (Esc / ctrl-c), returns an empty vec.
///
/// # Stdin behavior
///
/// fzf always reads interactive input from `/dev/tty`, so this works even when
/// equip's own stdin is a pipe. The items are fed to fzf's stdin via a pipe.
pub fn run_fzf(items: &[FzfItem], opts: &FzfOptions) -> Result<Vec<FzfItem>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    // We need a terminal for fzf interaction.
    // Note: fzf itself opens /dev/tty, so stdin being a pipe is fine.
    // But if there's truly no controlling terminal, bail.
    #[cfg(unix)]
    {
        if std::fs::metadata("/dev/tty").is_err() {
            bail!("fzf requires a terminal (no /dev/tty available)");
        }
    }

    let has_data = items.iter().any(|item| item.data.is_some());

    let mut args: Vec<String> = vec!["--ansi".to_string()];

    if opts.multi {
        args.push("--multi".to_string());
    }

    if has_data {
        args.extend(["--delimiter=\t".to_string(), "--with-nth=1".to_string()]);
    }

    if let Some(ref header) = opts.header {
        args.push(format!("--header={}", header));
    }

    if let Some(ref preview) = opts.preview {
        args.push(format!("--preview={}", preview));
    }

    if let Some(ref pw) = opts.preview_window {
        args.push(format!("--preview-window={}", pw));
    }

    if let Some(ref prompt) = opts.prompt {
        args.push(format!("--prompt={}", prompt));
    }

    for bind in &opts.bind {
        args.push(format!("--bind={}", bind));
    }

    // Build input text
    let input: String = items
        .iter()
        .map(|item| match &item.data {
            Some(data) => format!("{}\t{}", item.display, data),
            None => item.display.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("fzf")
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "fzf not found in PATH. Install fzf: https://github.com/junegunn/fzf"
                )
            } else {
                anyhow::anyhow!("failed to spawn fzf: {}", e)
            }
        })?;

    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(input.as_bytes());
    }
    drop(child.stdin.take());

    let output = child.wait_with_output()?;

    // Exit code 130 = user cancelled (ctrl-c), 1 = no match
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let selected: Vec<FzfItem> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            if has_data {
                let mut parts = line.splitn(2, '\t');
                let display = parts.next().unwrap_or("").to_string();
                let data = parts.next().map(|s| s.to_string());
                FzfItem { display, data }
            } else {
                FzfItem {
                    display: line.to_string(),
                    data: None,
                }
            }
        })
        .collect();

    Ok(selected)
}

/// Read skill identities from stdin when it is piped.
/// Returns None if stdin is a TTY.
pub fn read_stdin_items() -> Option<Vec<FzfItem>> {
    if std::io::stdin().is_terminal() {
        return None;
    }

    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return None;
    }

    let items: Vec<FzfItem> = buf
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| FzfItem {
            display: line.to_string(),
            data: None,
        })
        .collect();

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

/// Build FzfItems from skill tuples, attaching the SKILL.md path as data for preview.
pub fn skills_to_fzf_items(
    skills: &[(&str, &crate::registry::RegisteredPlugin, &crate::registry::RegisteredSkill)],
) -> Vec<FzfItem> {
    skills
        .iter()
        .map(|(source_name, plugin, skill)| {
            let identity =
                crate::output::plain_identity(source_name, &plugin.name, &skill.name);
            let skill_md = skill.path.join("SKILL.md");
            FzfItem {
                display: identity,
                data: Some(skill_md.display().to_string()),
            }
        })
        .collect()
}

/// Default FzfOptions for browsing skills.
pub fn skill_browse_options(multi: bool) -> FzfOptions {
    FzfOptions {
        multi,
        header: Some(if multi {
            "TAB to select, ENTER to confirm".to_string()
        } else {
            "ENTER to select".to_string()
        }),
        preview: Some("cat {2}".to_string()),
        preview_window: Some("right:60%:wrap".to_string()),
        prompt: Some("skill> ".to_string()),
        bind: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fzf_item_display_only() {
        let item = FzfItem {
            display: "test".to_string(),
            data: None,
        };
        assert_eq!(item.display, "test");
        assert!(item.data.is_none());
    }

    #[test]
    fn fzf_item_with_data() {
        let item = FzfItem {
            display: "src:plugin/skill".to_string(),
            data: Some("/path/to/SKILL.md".to_string()),
        };
        assert_eq!(item.display, "src:plugin/skill");
        assert_eq!(item.data.as_deref(), Some("/path/to/SKILL.md"));
    }

    #[test]
    fn default_options() {
        let opts = FzfOptions::default();
        assert!(!opts.multi);
        assert!(opts.header.is_none());
        assert!(opts.preview.is_none());
        assert!(opts.bind.is_empty());
    }

    #[test]
    fn skill_browse_options_single() {
        let opts = skill_browse_options(false);
        assert!(!opts.multi);
        assert!(opts.header.as_ref().unwrap().contains("ENTER"));
    }

    #[test]
    fn skill_browse_options_multi() {
        let opts = skill_browse_options(true);
        assert!(opts.multi);
        assert!(opts.header.as_ref().unwrap().contains("TAB"));
    }

    #[test]
    fn run_fzf_empty_items_returns_empty() {
        let result = run_fzf(&[], &FzfOptions::default()).unwrap();
        assert!(result.is_empty());
    }
}
