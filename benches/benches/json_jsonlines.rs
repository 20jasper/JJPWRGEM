use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jjpwrgem_parse::{format::LineEnding, jsonlines};

fn bench_jsonlines_format(c: &mut Criterion) {
    let data = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/logs.jsonl"),
    )
    .expect("logs.jsonl missing");

    let mut group = c.benchmark_group("jsonlines_format");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("jjpwrgem", "logs"), |b| {
        b.iter(|| jsonlines::format(black_box(&data), LineEnding::Lf).unwrap())
    });
    group.finish();
}

criterion_group!(benches, bench_jsonlines_format);
criterion_main!(benches);
