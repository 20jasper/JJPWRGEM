use clap::{Parser, Subcommand};

macro_rules! repo_path {
    ($path:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path)
    };
}

mod badges;
mod bench_table;
mod criterion;
mod generated;
mod lsp_bench_report;
mod markdown;
mod npm;
mod plot;
mod reporter_schema;
mod settings_docs;
mod template;
mod vscode_extension;
mod vscode_settings;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(clap::Args)]
struct CheckArgs {
    /// Fail if the committed output differs instead of writing it
    #[arg(long)]
    check: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate README.md files from templates
    Readmes(#[command(flatten)] CheckArgs),
    /// Write the format settings into the VS Code extension package.json
    VscodeSettings(#[command(flatten)] CheckArgs),
    /// Generate candlestick charts for all benchmark datasets
    PlotBenchmarks,
    /// Write the JSON Schema for the json reporter output
    ReporterSchema(#[command(flatten)] CheckArgs),
    /// Generate the npm package.json
    NpmPackage(#[command(flatten)] CheckArgs),
    /// Read criterion JSON from stdin and write a markdown table with time and throughput
    BenchTable,
    /// Read the lsp bench's criterion JSON from stdin plus its RSS sidecar and write per-fixture tables
    LspBenchReport,
    /// Print shields.io spec coverage badges for all tracked specs
    SpecCoverageBadges,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Readmes(CheckArgs { check }) => template::readmes(check),
        Commands::VscodeSettings(CheckArgs { check }) => vscode_settings::vscode_settings(check),
        Commands::ReporterSchema(CheckArgs { check }) => reporter_schema::reporter_schema(check),
        Commands::PlotBenchmarks => plot::plot_all_benchmarks(),
        Commands::NpmPackage(CheckArgs { check }) => npm::npm_package(check),
        Commands::BenchTable => bench_table::write_bench_table(),
        Commands::LspBenchReport => lsp_bench_report::write_lsp_bench_report(),
        Commands::SpecCoverageBadges => badges::write_spec_badges(),
    }
}
