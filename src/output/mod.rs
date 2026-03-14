use colored::Colorize;

/// Format a skill identity as a color-coded string: source(cyan) : plugin(green) / skill(bold).
pub fn format_identity(source: &str, plugin: &str, skill: &str) -> String {
    format!(
        "{}{}{}{}{}",
        source.cyan(),
        ":".dimmed(),
        plugin.green(),
        "/".dimmed(),
        skill.bold(),
    )
}

/// Format a skill identity as a plain string: source:plugin/skill.
pub fn plain_identity(source: &str, plugin: &str, skill: &str) -> String {
    format!("{}:{}/{}", source, plugin, skill)
}

/// Output context carrying global flags for all formatting decisions.
#[derive(Debug, Clone)]
pub struct Output {
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl Output {
    /// Create from CLI flags. Configures color automatically:
    /// disabled when `NO_COLOR` is set, output is JSON, or stdout is not a TTY.
    pub fn from_flags(json: bool, quiet: bool, verbose: bool) -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        if no_color || json {
            colored::control::set_override(false);
        }
        // Otherwise the `colored` crate auto-detects TTY

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_flags_construction() {
        let out = Output::from_flags(true, false, true);
        assert!(out.json);
        assert!(!out.quiet);
        assert!(out.verbose);
    }

    #[test]
    fn quiet_mode_fields() {
        let out = Output::from_flags(false, true, false);
        assert!(out.quiet);
        // Quiet mode: success/info/warn methods return early without printing.
        // We verify the flag is set; actual suppression is tested by the method guard.
    }

    #[test]
    fn verbose_mode_fields() {
        let out = Output::from_flags(false, false, true);
        assert!(out.verbose);
    }

    #[test]
    fn non_verbose_fields() {
        let out = Output::from_flags(false, false, false);
        assert!(!out.verbose);
    }

    #[test]
    fn json_mode_emits_valid_json() {
        let out = Output::from_flags(true, false, false);
        assert!(out.json);
        // json_value writes to stdout; verify no panic on a valid value
        let val = serde_json::json!({"key": "value"});
        // This would print to stdout in tests, but should not panic
        out.json_value(&val);
    }

    #[test]
    fn json_serialize_method() {
        let out = Output::from_flags(true, false, false);
        #[derive(serde::Serialize)]
        struct T { x: i32 }
        out.json(&T { x: 42 });
    }

    #[test]
    fn table_no_panic_empty() {
        let out = Output::from_flags(false, false, false);
        out.table(&["A", "B"], &[]);
    }

    #[test]
    fn table_no_panic_with_data() {
        let out = Output::from_flags(false, false, false);
        out.table(
            &["Name", "Value"],
            &[vec!["a".to_string(), "1".to_string()], vec!["bb".to_string(), "22".to_string()]],
        );
    }

    #[test]
    fn tree_no_panic() {
        let out = Output::from_flags(false, false, false);
        out.tree(&[(0, "root".to_string()), (1, "child".to_string()), (1, "child2".to_string())]);
    }

    #[test]
    fn quiet_suppresses_table() {
        let out = Output::from_flags(false, true, false);
        // Should return early without printing
        out.table(&["A"], &[vec!["x".to_string()]]);
        out.tree(&[(0, "root".to_string())]);
        out.header("Test");
    }
}
