use clap::Parser;
use loadout::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = loadout::cli::run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
