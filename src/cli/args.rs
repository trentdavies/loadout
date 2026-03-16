/// Preprocess raw CLI args to support shorthand syntax:
/// - `@name` → `--agent name`
/// - `+name` → `--kit name`
/// - Top-level catch-all: `loadout @claude ...` → `loadout agent equip @claude ...`
pub fn preprocess(raw: Vec<String>) -> Vec<String> {
    if raw.len() < 2 {
        return raw;
    }

    let mut result = Vec::with_capacity(raw.len() + 4);
    result.push(raw[0].clone());

    // Pass 1: consume leading global flags, detect catch-all
    let global_flags_no_arg = ["-n", "--dry-run", "-v", "--verbose", "-q", "--quiet", "--json"];
    let global_flags_with_arg = ["--config"];

    let mut i = 1;
    let mut global_prefix = Vec::new();

    while i < raw.len() {
        let arg = &raw[i];
        if global_flags_no_arg.contains(&arg.as_str()) {
            global_prefix.push(arg.clone());
            i += 1;
        } else if global_flags_with_arg.contains(&arg.as_str()) {
            global_prefix.push(arg.clone());
            i += 1;
            if i < raw.len() {
                global_prefix.push(raw[i].clone());
                i += 1;
            }
        } else {
            break;
        }
    }

    // Check if first positional starts with @ or +
    let needs_catchall = i < raw.len() && (raw[i].starts_with('@') || raw[i].starts_with('+'));

    result.extend(global_prefix);

    if needs_catchall {
        result.push("_equip".to_string());
    }

    // Pass 2: expand shorthands if subcommand is agent {equip, unequip, collect}
    // To avoid clap's greedy num_args consuming positional patterns as flag values,
    // we collect expanded flags and emit them after all other args.
    let rest = &raw[i..];
    let subcommand = detect_subcommand(&result, rest);

    let mut trailing_flags: Vec<String> = Vec::new();
    let mut j = 0;
    let mut past_double_dash = false;
    while j < rest.len() {
        let arg = &rest[j];

        if arg == "--" {
            past_double_dash = true;
            result.push(arg.clone());
            j += 1;
            continue;
        }

        if past_double_dash {
            result.push(arg.clone());
            j += 1;
            continue;
        }

        match subcommand {
            Some(Sub::Equip) => {
                if let Some(name) = arg.strip_prefix('@') {
                    trailing_flags.push("--agent".to_string());
                    trailing_flags.push(strip_quotes(name));
                } else if let Some(name) = arg.strip_prefix('+') {
                    trailing_flags.push("--kit".to_string());
                    trailing_flags.push(strip_quotes(name));
                } else {
                    result.push(arg.clone());
                }
            }
            Some(Sub::Collect) => {
                if let Some(name) = arg.strip_prefix('@') {
                    trailing_flags.push("--agent".to_string());
                    trailing_flags.push(strip_quotes(name));
                } else {
                    result.push(arg.clone());
                }
            }
            None => {
                result.push(arg.clone());
            }
        }
        j += 1;
    }

    result.extend(trailing_flags);
    result
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Sub {
    Equip,
    Collect,
}

/// Detect if the resolved args form a `_equip` or `agent collect` subcommand path.
fn detect_subcommand(prefix: &[String], rest: &[String]) -> Option<Sub> {
    let all: Vec<&str> = prefix.iter().chain(rest.iter()).map(|s| s.as_str()).collect();
    let flags_with_arg = ["--config"];

    let mut found_agent = false;
    let mut i = 1; // skip program name
    while i < all.len() {
        let token = all[i];
        if token.starts_with('-') {
            // skip flags, consuming value for flags that take one
            if flags_with_arg.contains(&token) {
                i += 1; // skip the value
            }
            i += 1;
            continue;
        }
        if token.starts_with('@') || token.starts_with('+') {
            // shorthand tokens aren't subcommands — skip them
            i += 1;
            continue;
        }
        // Top-level _equip
        if token == "_equip" {
            return Some(Sub::Equip);
        }
        // agent collect path
        if !found_agent {
            if token == "agent" {
                found_agent = true;
            } else {
                return None;
            }
        } else {
            return match token {
                "collect" => Some(Sub::Collect),
                _ => None,
            };
        }
        i += 1;
    }
    None
}

fn strip_quotes(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pp(args: &[&str]) -> Vec<String> {
        preprocess(args.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn at_agent_in_equip() {
        let result = pp(&["loadout", "_equip", "@claude", "dev*"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "dev*", "--agent", "claude"]
        );
    }

    #[test]
    fn plus_kit_in_equip() {
        let result = pp(&["loadout", "_equip", "+developer"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "--kit", "developer"]
        );
    }

    #[test]
    fn multiple_at_args() {
        let result = pp(&["loadout", "_equip", "@claude", "@cursor"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "--agent", "claude", "--agent", "cursor"]
        );
    }

    #[test]
    fn top_level_catchall_with_at() {
        let result = pp(&["loadout", "@claude", "dev*"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "dev*", "--agent", "claude"]
        );
    }

    #[test]
    fn top_level_catchall_with_plus() {
        let result = pp(&["loadout", "+dev"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "--kit", "dev"]
        );
    }

    #[test]
    fn global_flags_before_catchall() {
        let result = pp(&["loadout", "-n", "--verbose", "@claude", "dev*"]);
        assert_eq!(
            result,
            [
                "loadout", "-n", "--verbose", "_equip", "dev*", "--agent", "claude"
            ]
        );
    }

    #[test]
    fn config_flag_consumes_two_tokens() {
        let result = pp(&["loadout", "--config", "/tmp/alt.toml", "@claude"]);
        assert_eq!(
            result,
            [
                "loadout", "--config", "/tmp/alt.toml", "_equip", "--agent", "claude"
            ]
        );
    }

    #[test]
    fn no_expansion_in_list() {
        let result = pp(&["loadout", "list", "@something"]);
        assert_eq!(result, ["loadout", "list", "@something"]);
    }

    #[test]
    fn agent_collect_expands_at_not_plus() {
        let result = pp(&["loadout", "agent", "collect", "@claude", "+dev"]);
        assert_eq!(
            result,
            ["loadout", "agent", "collect", "+dev", "--agent", "claude"]
        );
    }

    #[test]
    fn double_dash_stops_expansion() {
        let result = pp(&["loadout", "_equip", "@claude", "--", "+notkit"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "--", "+notkit", "--agent", "claude"]
        );
    }

    #[test]
    fn quoted_values_stripped() {
        let result = pp(&["loadout", "_equip", "@\"my-agent\""]);
        assert_eq!(
            result,
            ["loadout", "_equip", "--agent", "my-agent"]
        );
    }

    #[test]
    fn multiple_globs_preserved() {
        let result = pp(&["loadout", "@claude", "dev*", "legal/*"]);
        assert_eq!(
            result,
            ["loadout", "_equip", "dev*", "legal/*", "--agent", "claude"]
        );
    }

    #[test]
    fn full_scenario_with_save() {
        let result = pp(&["loadout", "@claude", "+developer", "-s", "dev*", "legal/*"]);
        assert_eq!(
            result,
            [
                "loadout", "_equip", "-s", "dev*", "legal/*",
                "--agent", "claude", "--kit", "developer"
            ]
        );
    }

    #[test]
    fn no_args_passthrough() {
        let result = pp(&["loadout"]);
        assert_eq!(result, ["loadout"]);
    }

    #[test]
    fn regular_subcommand_no_catchall() {
        let result = pp(&["loadout", "status"]);
        assert_eq!(result, ["loadout", "status"]);
    }
}
