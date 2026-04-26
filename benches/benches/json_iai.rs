use std::hint::black_box;

use gungraun::{
    Callgrind, CallgrindMetrics, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};
use jjpwrgem_parse::{
    ast::{Value, parse_str},
    format::{LineEnding, prettify_value_into, uglify_str},
};

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

fn setup_citm_ast() -> Value<'static> {
    parse_str(CITM).unwrap()
}

fn setup_canada_ast() -> Value<'static> {
    parse_str(CANADA).unwrap()
}

fn setup_twitter_ast() -> Value<'static> {
    parse_str(TWITTER).unwrap()
}

#[library_benchmark]
#[bench::citm(setup = setup_citm_ast)]
#[bench::canada(setup = setup_canada_ast)]
#[bench::twitter(setup = setup_twitter_ast)]
fn bench_prettify_ast(ast: Value<'static>) -> String {
    let mut buf = String::new();
    prettify_value_into(&mut buf, black_box(&ast), 80, LineEnding::Lf);
    buf
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

library_benchmark_group!(
    name = prettify_ast_group;
    config = branch_sim_config();
    benchmarks = bench_prettify_ast
);

main!(
    library_benchmark_groups = deser_group,
    uglify_tokens_group,
    prettify_ast_group
);
