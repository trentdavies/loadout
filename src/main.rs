use clap::Parser;
use loadout::cli::Cli;

fn main() {
    let raw: Vec<String> = std::env::args().collect();
    let processed = loadout::cli::args::preprocess(raw);
    let cli = Cli::parse_from(processed);

    if let Err(e) = loadout::cli::run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
