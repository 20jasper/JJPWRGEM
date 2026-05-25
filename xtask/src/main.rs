use clap::{Parser, Subcommand};

mod bench_table;
mod lsp_bench_report;
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
    /// Read criterion JSON from stdin and write a markdown table with time and throughput
    BenchTable,
    /// Read lsp-bench results.json and write a markdown table for the named fixture
    LspBenchReport { name: String },
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
        Commands::BenchTable => bench_table::write_bench_table(),
        Commands::LspBenchReport { name } => lsp_bench_report::write_lsp_bench_report(&name),
    }
}
