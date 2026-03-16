use clap::Parser;
use loadout::cli::Cli;

fn main() {
    let raw: Vec<String> = std::env::args().collect();

    // Block direct invocation of internal commands
    if raw.iter().skip(1).any(|a| a == "_equip") {
        eprintln!("error: '_equip' is an internal command. Use @agent/+kit shorthand instead.");
        eprintln!("  loadout @claude dev*           # equip skills to claude");
        eprintln!("  loadout +dev                   # equip kit to auto-sync agents");
        eprintln!("  loadout @claude +dev --remove  # unequip");
        std::process::exit(2);
    }

    let processed = loadout::cli::args::preprocess(raw);
    let cli = Cli::parse_from(processed);

    if let Err(e) = loadout::cli::run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
