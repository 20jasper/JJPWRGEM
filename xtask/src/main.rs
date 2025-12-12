use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod plot;
mod template;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate README.md files from templates
    GenerateReadmes,
    /// Verify generated READMEs match what's committed
    VerifyReadmes,
    /// Generate candlestick charts for all benchmark datasets
    PlotBenchmarks,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::GenerateReadmes => {
            template::write_readmes();
            println!("Wrote README files");
        }
        Commands::VerifyReadmes => {
            if let Err(e) = template::are_readmes_updated() {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::PlotBenchmarks => match plot::plot_all_benchmarks() {
            Ok(()) => println!("Wrote candlestick charts for all benchmarks"),
            Err(err) => {
                eprintln!("failed to plot candlestick charts: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    ExitCode::SUCCESS
}
