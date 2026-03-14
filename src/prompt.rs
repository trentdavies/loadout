use anyhow::{bail, Result};
use std::io::IsTerminal;

/// Returns true when stdin is a TTY and interactive prompts can be shown.
pub fn is_interactive() -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_interactive_false_in_tests() {
        // Test runners pipe stdin, so this should always be false.
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
    fn select_from_errors_non_interactive() {
        let options = vec!["alpha".to_string(), "beta".to_string()];
        let result = select_from("Source", &options, false);
        assert!(result.is_err());
    }

    #[test]
    fn select_from_errors_quiet() {
        let options = vec!["alpha".to_string(), "beta".to_string()];
        let result = select_from("Source", &options, true);
        assert!(result.is_err());
    }
}
