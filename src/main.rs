use clap::Parser;
use forge-te::cli::Cli;

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
