use anyhow::{bail, Result};
use std::io::IsTerminal;

/// Returns true when stdin is a TTY and interactive prompts can be shown.
/// Respects `SKITTLE_NON_INTERACTIVE=1` to force non-interactive mode (used in tests).
pub fn is_interactive() -> bool {
    if std::env::var("SKITTLE_NON_INTERACTIVE").is_ok() {
        return false;
    }
    std::io::stdin().is_terminal()
}

/// Show `label` with a `default` value; return the default on Enter or the user's override.
/// In non-interactive or quiet mode, returns the default without prompting.
pub fn confirm_or_override(label: &str, default: &str, quiet: bool) -> String {
    if quiet || !is_interactive() {
        return default.to_string();
    }

    dialoguer::Input::<String>::new()
        .with_prompt(label)
        .default(default.to_string())
        .interact_text()
        .unwrap_or_else(|_| default.to_string())
}

/// Display a numbered list of `options` and return the selected value.
/// Errors in non-interactive or quiet mode since a selection cannot be inferred.
pub fn select_from(label: &str, options: &[String], quiet: bool) -> Result<String> {
    if quiet || !is_interactive() {
        bail!("interactive input required for selection (not a TTY or --quiet)");
    }

    let idx = dialoguer::Select::new()
        .with_prompt(label)
        .items(options)
        .default(0)
        .interact()?;

    Ok(options[idx].clone())
}

/// Present a multi-select list. Returns indices of selected items.
/// In non-interactive or quiet mode, returns an empty vec.
pub fn multi_select(label: &str, options: &[&str], defaults: &[bool], quiet: bool) -> Vec<usize> {
    if quiet || !is_interactive() {
        return Vec::new();
    }

    dialoguer::MultiSelect::new()
        .with_prompt(label)
        .items(options)
        .defaults(defaults)
        .interact()
        .unwrap_or_default()
}

/// Prompt for local source fetch mode: symlink (default) or copy.
/// Returns "symlink" or "copy". In non-interactive/quiet mode, returns "symlink".
pub fn prompt_fetch_mode(quiet: bool) -> String {
    if quiet || !is_interactive() {
        return "symlink".to_string();
    }

    let options = &["symlink (live edits)", "copy (snapshot)"];
    let idx = dialoguer::Select::new()
        .with_prompt("Fetch mode")
        .items(options)
        .default(0)
        .interact()
        .unwrap_or(0);

    if idx == 0 {
        "symlink".to_string()
    } else {
        "copy".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_interactive_false_when_env_set() {
        std::env::set_var("SKITTLE_NON_INTERACTIVE", "1");
        assert!(!is_interactive());
    }

    #[test]
    fn confirm_or_override_returns_default_non_interactive() {
        let result = confirm_or_override("Source", "my-skills", false);
        assert_eq!(result, "my-skills");
    }

    #[test]
    fn confirm_or_override_returns_default_quiet() {
        let result = confirm_or_override("Source", "my-skills", true);
        assert_eq!(result, "my-skills");
    }

    #[test]
    fn multi_select_returns_empty_non_interactive() {
        let result = multi_select("Pick", &["a", "b"], &[true, true], false);
        assert!(result.is_empty());
    }

    #[test]
    fn multi_select_returns_empty_quiet() {
        let result = multi_select("Pick", &["a", "b"], &[true, true], true);
        assert!(result.is_empty());
    }

    #[test]
    fn select_from_errors_non_interactive() {
        let options = vec!["alpha".to_string(), "beta".to_string()];
        let result = select_from("Source", &options, false);
        assert!(result.is_err());
    }

    #[test]
    fn fetch_mode_returns_symlink_non_interactive() {
        let result = prompt_fetch_mode(false);
        assert_eq!(result, "symlink");
    }

    #[test]
    fn fetch_mode_returns_symlink_quiet() {
        let result = prompt_fetch_mode(true);
        assert_eq!(result, "symlink");
    }

    #[test]
    fn select_from_errors_quiet() {
        let options = vec!["alpha".to_string(), "beta".to_string()];
        let result = select_from("Source", &options, true);
        assert!(result.is_err());
    }
}
