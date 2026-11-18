use clap::{Parser, Subcommand};

macro_rules! repo_path {
    ($path:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path)
    };
}

macro_rules! template {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/", $name))
    };
}

mod badge;
mod bench_table;
mod generated;
mod lsp_bench_report;
mod npm;
mod plot;
mod settings_docs;
mod spec_badges;
mod template;
mod vscode_settings;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate README.md files from templates
    Readmes {
        /// Fail if the committed files differ instead of writing them
        #[arg(long)]
        check: bool,
    },
    /// Write the format settings into the VS Code extension package.json
    VscodeSettings {
        /// Fail if the committed file differs instead of writing it
        #[arg(long)]
        check: bool,
    },
    /// Generate candlestick charts for all benchmark datasets
    PlotBenchmarks,
    /// Generate the npm package.json
    NpmPackage {
        /// Fail if the committed file differs instead of writing it
        #[arg(long)]
        check: bool,
    },
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
        Commands::Readmes { check } => template::readmes(check),
        Commands::VscodeSettings { check } => vscode_settings::vscode_settings(check),
        Commands::PlotBenchmarks => plot::plot_all_benchmarks(),
        Commands::NpmPackage { check } => npm::npm_package(check),
        Commands::BenchTable => bench_table::write_bench_table(),
        Commands::LspBenchReport => lsp_bench_report::write_lsp_bench_report(),
        Commands::SpecCoverageBadges => spec_badges::write_spec_badges(),
    }
}
