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

    /// Print a table with headers and rows.
    pub fn table(&self, headers: &[&str], rows: &[Vec<String>]) {
        if self.quiet { return; }

        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print header
        let header_line: Vec<String> = headers.iter().enumerate()
            .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
            .collect();
        println!("{}", header_line.join("  ").bold());

        // Print separator
        let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
        println!("{}", sep.join("  ").dimmed());

        // Print rows
        for row in rows {
            let line: Vec<String> = row.iter().enumerate()
                .map(|(i, cell)| {
                    let w = widths.get(i).copied().unwrap_or(cell.len());
                    format!("{:<width$}", cell, width = w)
                })
                .collect();
            println!("{}", line.join("  "));
        }
    }

    /// Print a tree structure. Each entry is (depth, label).
    pub fn tree(&self, entries: &[(usize, String)]) {
        if self.quiet { return; }

        for (i, (depth, label)) in entries.iter().enumerate() {
            let is_last = entries.get(i + 1)
                .map(|(d, _)| *d <= *depth)
                .unwrap_or(true);

            let mut prefix = String::new();
            for d in 0..*depth {
                if d == depth - 1 {
                    prefix.push_str(if is_last { "└── " } else { "├── " });
                } else {
                    // Check if any later sibling exists at this ancestor depth
                    let has_sibling = entries[i + 1..].iter()
                        .any(|(fd, _)| *fd <= d);
                    prefix.push_str(if has_sibling { "│   " } else { "    " });
                }
            }
            println!("{}{}", prefix.dimmed(), label);
        }
    }

    /// Print a section header.
    pub fn header(&self, title: &str) {
        if self.quiet { return; }
        println!();
        println!("{}", title.bold().underline());
    }
}
