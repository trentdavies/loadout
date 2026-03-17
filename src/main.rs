use clap::Parser;
use equip::cli::Cli;

fn main() {
    let raw: Vec<String> = std::env::args().collect();

    // Block direct invocation of internal commands
    if raw.iter().skip(1).any(|a| a == "_equip") {
        eprintln!("error: '_equip' is an internal command. Use @agent/+kit shorthand instead.");
        eprintln!("  equip @claude dev*           # equip skills to claude");
        eprintln!("  equip +dev                   # equip kit to auto-sync agents");
        eprintln!("  equip @claude +dev --remove  # unequip");
        std::process::exit(2);
    }

    let processed = equip::cli::args::preprocess(raw);
    let cli = Cli::parse_from(processed);

    if let Err(e) = equip::cli::run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
