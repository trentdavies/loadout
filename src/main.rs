use clap::Parser;
use skittle::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = skittle::cli::run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
