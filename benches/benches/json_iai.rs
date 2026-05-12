use std::hint::black_box;

use gungraun::{
    Callgrind, CallgrindMetrics, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};
use jjpwrgem_parse::{
    ast::Document,
    format::{LineEnding, prettify_document_into, uglify_str},
    tokens::TokenStream,
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
fn bench_deser(json: &'static str) -> Document<&'static str> {
    Document::parse(black_box(json)).unwrap()
}

#[library_benchmark]
#[bench::citm(CITM)]
#[bench::canada(CANADA)]
#[bench::twitter(TWITTER)]
fn bench_uglify_tokens(json: &'static str) -> String {
    uglify_str(black_box(json)).unwrap()
}

fn setup_citm_ast() -> Document<&'static str> {
    Document::parse(CITM).unwrap()
}

fn setup_canada_ast() -> Document<&'static str> {
    Document::parse(CANADA).unwrap()
}

fn setup_twitter_ast() -> Document<&'static str> {
    Document::parse(TWITTER).unwrap()
}

#[library_benchmark]
#[bench::citm(setup = setup_citm_ast)]
#[bench::canada(setup = setup_canada_ast)]
#[bench::twitter(setup = setup_twitter_ast)]
fn bench_prettify_ast(doc: Document<&'static str>) -> String {
    let mut buf = String::new();
    prettify_document_into(&mut buf, black_box(&doc), 80, LineEnding::Lf);
    buf
}

#[library_benchmark]
#[bench::citm(CITM)]
#[bench::canada(CANADA)]
#[bench::twitter(TWITTER)]
fn bench_tokens_plain(json: &'static str) {
    TokenStream::new(black_box(json)).for_each(|token| {
        black_box(token.unwrap());
    })
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

library_benchmark_group!(
    name = tokens_plain_group;
    config = branch_sim_config();
    benchmarks = bench_tokens_plain
);

main!(
    library_benchmark_groups = deser_group,
    uglify_tokens_group,
    prettify_ast_group,
    tokens_plain_group
);
