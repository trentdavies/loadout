use colored::Colorize;

use crate::cli::ColorWhen;

/// Output context carrying global flags for all formatting decisions.
#[derive(Debug, Clone)]
pub struct Output {
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl Output {
    /// Create from CLI flags. Also configures the `colored` crate globally.
    pub fn from_flags(json: bool, quiet: bool, verbose: bool, color: &ColorWhen) -> Self {
        // Configure colored crate based on --color flag and NO_COLOR env
        let no_color = std::env::var("NO_COLOR").is_ok();
        match color {
            ColorWhen::Never => colored::control::set_override(false),
            ColorWhen::Always => colored::control::set_override(true),
            ColorWhen::Auto => {
                if no_color || json {
                    colored::control::set_override(false);
                }
                // Otherwise let colored auto-detect TTY
            }
        }

        Self { json, quiet, verbose }
    }

    /// Print a success message (green checkmark).
    pub fn success(&self, msg: &str) {
        if self.quiet { return; }
        println!("{} {}", "✓".green(), msg);
    }

    /// Print a warning message (yellow).
    pub fn warn(&self, msg: &str) {
        if self.quiet { return; }
        eprintln!("{} {}", "warning:".yellow(), msg);
    }

    /// Print an error message (red) to stderr.
    pub fn error(&self, msg: &str) {
        eprintln!("{} {}", "error:".red(), msg);
    }

    /// Print an info message (only if not quiet).
    pub fn info(&self, msg: &str) {
        if self.quiet { return; }
        println!("{}", msg);
    }

    /// Print a verbose/debug message (only if --verbose).
    pub fn debug(&self, msg: &str) {
        if !self.verbose { return; }
        println!("{} {}", "debug:".dimmed(), msg);
    }

    /// Print a status line: label in bold, value normal.
    pub fn status(&self, label: &str, value: &str) {
        if self.quiet { return; }
        println!("{} {}", format!("{}:", label).bold(), value);
    }

    /// Print JSON value and return Ok.
    pub fn json_value(&self, value: &serde_json::Value) {
        println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
    }

    /// Print a serializable value as JSON.
    pub fn json<T: serde::Serialize>(&self, value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{}", json);
        }
    }
}
