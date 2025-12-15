use clap::{Parser, Subcommand};

mod npm;
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
    /// Generate npm package.json
    GenerateNpmPackage,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::GenerateReadmes => {
            template::write_readmes();
            Ok(())
        }
        Commands::VerifyReadmes => template::are_readmes_updated(),
        Commands::PlotBenchmarks => plot::plot_all_benchmarks(),
        Commands::GenerateNpmPackage => npm::write_package_json(),
    }
}
