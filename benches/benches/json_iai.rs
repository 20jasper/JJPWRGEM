use std::hint::black_box;

use gungraun::{
    Callgrind, CallgrindMetrics, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};
use jjpwrgem_parse::{ast::parse_str, format::uglify_str};

const CITM: &str = include_str!("../../xtask/bench/data/json-benchmark/data/citm_catalog.json");
const CANADA: &str = include_str!("../../xtask/bench/data/json-benchmark/data/canada.json");
const TWITTER: &str = include_str!("../../xtask/bench/data/json-benchmark/data/twitter.json");

fn branch_sim_config() -> LibraryBenchmarkConfig {
    let mut config = LibraryBenchmarkConfig::default();
    config.tool(
        Callgrind::with_args(["--branch-sim=yes"])
            .format([CallgrindMetrics::Default, CallgrindMetrics::BranchSim]),
    );
    config
}

#[library_benchmark]
#[bench::citm(CITM)]
#[bench::canada(CANADA)]
#[bench::twitter(TWITTER)]
fn bench_deser(json: &'static str) -> jjpwrgem_parse::ast::Value<'static> {
    parse_str(black_box(json)).unwrap()
}

#[library_benchmark]
#[bench::citm(CITM)]
#[bench::canada(CANADA)]
#[bench::twitter(TWITTER)]
fn bench_uglify_tokens(json: &'static str) -> String {
    uglify_str(black_box(json)).unwrap()
}

library_benchmark_group!(
    name = deser_group;
    config = branch_sim_config();
    benchmarks = bench_deser
);

library_benchmark_group!(
    name = uglify_tokens_group;
    config = branch_sim_config();
    benchmarks = bench_uglify_tokens
);

main!(library_benchmark_groups = deser_group, uglify_tokens_group);
