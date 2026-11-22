use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

mod json_common;

use jjpwrgem_parse::{SyntaxOptions, ast::Document};
use json_common::{include_impl, load_inputs};

const COMMENT_AFTER_LINE: &str = " // comment\n";

const LONG_COMMENT_LEN: usize = 5_000;
const LONG_COMMENT_COUNT: usize = 20;
const HUGE_COMMENT_LEN: usize = 1_000_000;

fn long_comments(len: usize, count: usize) -> String {
    let comment = format!("// {}\n", "x".repeat(len));
    let items = vec![format!("{comment}1"); count].join(",\n");
    format!("[\n{items}\n]")
}

fn load_jsonc_inputs() -> Vec<(&'static str, String)> {
    let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let commented_citm = std::fs::read_to_string(data.join("citm_catalog.json"))
        .unwrap()
        .replace('\n', COMMENT_AFTER_LINE);
    vec![
        (
            "tsconfig",
            std::fs::read_to_string(data.join("tsconfig.jsonc")).unwrap(),
        ),
        ("citm_catalog_commented", commented_citm),
        (
            "long_comments",
            long_comments(LONG_COMMENT_LEN, LONG_COMMENT_COUNT),
        ),
        ("huge_comment", long_comments(HUGE_COMMENT_LEN, 1)),
    ]
    .into_iter()
    .filter(|(name, _)| match std::env::var("JJP_JSON_INPUT") {
        Ok(filter) if !filter.is_empty() => filter == *name,
        _ => true,
    })
    .collect()
}

fn bench_deser(c: &mut Criterion) {
    let inputs = load_inputs();
    let mut group = c.benchmark_group("deser");

    for (name, json) in &inputs {
        group.throughput(Throughput::Bytes(json.len() as u64));

        if include_impl("jjpwrgem") {
            group.bench_with_input(
                BenchmarkId::new("jjpwrgem", name),
                json.as_str(),
                |b, json| {
                    b.iter(|| Document::parse(black_box(json), SyntaxOptions::JSON).unwrap());
                },
            );
        }

        if include_impl("serde_json") {
            group.bench_with_input(
                BenchmarkId::new("serde_json", name),
                json.as_str(),
                |b, json| {
                    b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json)).unwrap());
                },
            );
        }

        if include_impl("simd_json") {
            group.bench_with_input(
                BenchmarkId::new("simd_json", name),
                json.as_bytes(),
                |b, bytes| {
                    b.iter_batched(
                        || bytes.to_vec(),
                        |mut data| {
                            simd_json::to_owned_value(black_box(data.as_mut_slice())).unwrap()
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        if include_impl("sonic_rs") {
            group.bench_with_input(
                BenchmarkId::new("sonic_rs", name),
                json.as_str(),
                |b, json| {
                    b.iter(|| sonic_rs::from_str::<sonic_rs::Value>(black_box(json)).unwrap());
                },
            );
        }
    }

    if include_impl("jjpwrgem") {
        for (name, jsonc) in &load_jsonc_inputs() {
            group.throughput(Throughput::Bytes(jsonc.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("jjpwrgem_jsonc", name),
                jsonc.as_str(),
                |b, jsonc| {
                    b.iter(|| Document::parse(black_box(jsonc), SyntaxOptions::JSONC).unwrap());
                },
            );
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_deser
}
criterion_main!(benches);
