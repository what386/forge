use clap::Parser;
use forge_te::cli::Cli;

fn main() {
    if let Err(err) = Cli::parse().run() {
        #[cfg(debug_assertions)]
        {
            eprintln!("{:?}", err);
        }

        #[cfg(not(debug_assertions))]
        {
            eprintln!(
                "{}",
                err.chain()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        std::process::exit(1);
    }
}
