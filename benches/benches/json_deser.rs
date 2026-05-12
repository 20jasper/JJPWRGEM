use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

mod json_common;

use jjpwrgem_parse::ast::Document;
use json_common::{include_impl, load_inputs};

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
                    b.iter(|| Document::parse(black_box(json)).unwrap());
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

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_deser
}
criterion_main!(benches);
