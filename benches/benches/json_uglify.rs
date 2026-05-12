use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jjpwrgem_parse::{
    ast::Document,
    format::{uglify_document_into, uglify_str},
};
use simd_json::prelude::Writable as _;

mod json_common;

use json_common::{include_impl, load_inputs};

fn bench_uglify_ast(c: &mut Criterion, inputs: &[(&str, String)]) {
    let mut group = c.benchmark_group("uglify_ast");

    for (name, json) in inputs {
        // Normalize throughput by source size so all formatters are comparable.
        group.throughput(Throughput::Bytes(json.len() as u64));

        let ast = Document::parse(json.as_str()).unwrap();
        if include_impl("jjpwrgem") {
            let mut probe = String::new();
            uglify_document_into(&mut probe, &ast);
            let mut jjp_buf = String::with_capacity(probe.len());
            group.bench_function(BenchmarkId::new("jjpwrgem", name), |b| {
                b.iter(|| {
                    jjp_buf.clear();
                    uglify_document_into(&mut jjp_buf, black_box(&ast));
                    black_box(&jjp_buf);
                });
            });
        }

        if include_impl("serde_json") {
            let serde_val: serde_json::Value = serde_json::from_str(json).unwrap();
            let mut probe_vec = Vec::new();
            serde_json::to_writer(&mut probe_vec, &serde_val).unwrap();
            let mut serde_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("serde_json", name), |b| {
                b.iter(|| {
                    serde_buf.clear();
                    serde_json::to_writer(&mut serde_buf, black_box(&serde_val)).unwrap();
                    black_box(&serde_buf);
                });
            });
        }

        if include_impl("simd_json") {
            let mut owned_input = json.as_bytes().to_vec();
            let owned = simd_json::to_owned_value(owned_input.as_mut_slice()).unwrap();
            let mut probe_vec = Vec::new();
            owned.write(&mut probe_vec).unwrap();
            let mut simd_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("simd_json", name), |b| {
                b.iter(|| {
                    simd_buf.clear();
                    owned.write(black_box(&mut simd_buf)).unwrap();
                    black_box(&simd_buf);
                });
            });
        }

        if include_impl("sonic_rs") {
            let sonic_val: sonic_rs::Value = sonic_rs::from_str(json).unwrap();
            let mut probe_vec = Vec::new();
            sonic_rs::to_writer(&mut probe_vec, &sonic_val).unwrap();
            let mut sonic_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("sonic_rs", name), |b| {
                b.iter(|| {
                    sonic_buf.clear();
                    sonic_rs::to_writer(&mut sonic_buf, black_box(&sonic_val)).unwrap();
                    black_box(&sonic_buf);
                });
            });
        }
    }

    group.finish();
}

fn bench_uglify_tokens(c: &mut Criterion, inputs: &[(&str, String)]) {
    let mut group = c.benchmark_group("uglify_tokens");

    for (name, json) in inputs {
        group.throughput(Throughput::Bytes(json.len() as u64));
        if include_impl("jjpwrgem") {
            group.bench_with_input(
                BenchmarkId::new("jjpwrgem", name),
                json.as_str(),
                |b, json| {
                    b.iter(|| black_box(uglify_str(black_box(json)).unwrap()));
                },
            );
        }
    }

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    let inputs = load_inputs();
    bench_uglify_ast(c, &inputs);
    bench_uglify_tokens(c, &inputs);
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}
criterion_main!(benches);
