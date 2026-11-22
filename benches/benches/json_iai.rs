use std::{hint::black_box, sync::LazyLock};

use gungraun::{
    Callgrind, CallgrindMetrics, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};
use jjpwrgem_parse::{
    SyntaxOptions,
    ast::Document,
    format::{LineEnding, prettify_document_into, uglify_str},
    tokens::TokenStream,
};

const CITM: &str = include_str!("../data/citm_catalog.json");
const CANADA: &str = include_str!("../data/canada.json");
const TWITTER: &str = include_str!("../data/twitter.json");

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
    Document::parse(black_box(json), SyntaxOptions::JSON).unwrap()
}

fn setup_citm_line_comments() -> &'static str {
    static INPUT: LazyLock<String> = LazyLock::new(|| CITM.replace('\n', " // comment\n"));
    INPUT.as_str()
}

fn setup_citm_block_comments() -> &'static str {
    static INPUT: LazyLock<String> = LazyLock::new(|| CITM.replace('\n', " /* comment */\n"));
    INPUT.as_str()
}

#[library_benchmark]
#[bench::citm_line(setup = setup_citm_line_comments)]
#[bench::citm_block(setup = setup_citm_block_comments)]
fn bench_deser_jsonc(json: &'static str) -> Document<&'static str> {
    Document::parse(black_box(json), SyntaxOptions::JSONC).unwrap()
}

#[library_benchmark]
#[bench::citm(CITM)]
#[bench::canada(CANADA)]
#[bench::twitter(TWITTER)]
fn bench_uglify_tokens(json: &'static str) -> String {
    uglify_str(black_box(json)).unwrap()
}

fn setup_citm_ast() -> Document<&'static str> {
    Document::parse(CITM, SyntaxOptions::JSON).unwrap()
}

fn setup_canada_ast() -> Document<&'static str> {
    Document::parse(CANADA, SyntaxOptions::JSON).unwrap()
}

fn setup_twitter_ast() -> Document<&'static str> {
    Document::parse(TWITTER, SyntaxOptions::JSON).unwrap()
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
    TokenStream::new(black_box(json), SyntaxOptions::JSON).for_each(|token| {
        black_box(token.unwrap());
    })
}

library_benchmark_group!(
    name = deser_group;
    config = branch_sim_config();
    benchmarks = bench_deser
);

library_benchmark_group!(
    name = deser_jsonc_group;
    config = branch_sim_config();
    benchmarks = bench_deser_jsonc
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
    deser_jsonc_group,
    uglify_tokens_group,
    prettify_ast_group,
    tokens_plain_group
);
