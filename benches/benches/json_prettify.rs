use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jjpwrgem_parse::{
    ast::parse_str,
    format::{LineEnding, prettify_value_into},
};
use simd_json::prelude::Writable as _;

mod json_common;

use json_common::{include_impl, load_inputs};

fn bench_prettify_ast(c: &mut Criterion) {
    let inputs = load_inputs();
    let mut group = c.benchmark_group("prettify_ast");

    for (name, json) in &inputs {
        // Normalize throughput by source size so all formatters are comparable.
        group.throughput(Throughput::Bytes(json.len() as u64));

        let ast = parse_str(json).unwrap();
        if include_impl("jjpwrgem") {
            let mut probe = String::new();
            prettify_value_into(&mut probe, &ast, 80, LineEnding::Lf);
            let mut jjp_buf = String::with_capacity(probe.len());
            group.bench_function(BenchmarkId::new("jjpwrgem", name), |b| {
                b.iter(|| {
                    jjp_buf.clear();
                    prettify_value_into(&mut jjp_buf, black_box(&ast), 80, LineEnding::Lf);
                    black_box(&jjp_buf);
                });
            });
        }

        if include_impl("serde_json") {
            let serde_val: serde_json::Value = serde_json::from_str(json).unwrap();
            let mut probe_vec = Vec::new();
            serde_json::to_writer_pretty(&mut probe_vec, &serde_val).unwrap();
            let mut serde_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("serde_json", name), |b| {
                b.iter(|| {
                    serde_buf.clear();
                    serde_json::to_writer_pretty(&mut serde_buf, black_box(&serde_val)).unwrap();
                    black_box(&serde_buf);
                });
            });
        }

        if include_impl("simd_json") {
            let mut owned_input = json.as_bytes().to_vec();
            let owned = simd_json::to_owned_value(owned_input.as_mut_slice()).unwrap();
            let mut probe_vec = Vec::new();
            owned.write_pp(&mut probe_vec).unwrap();
            let mut simd_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("simd_json", name), |b| {
                b.iter(|| {
                    simd_buf.clear();
                    owned.write_pp(black_box(&mut simd_buf)).unwrap();
                    black_box(&simd_buf);
                });
            });
        }

        if include_impl("sonic_rs") {
            let sonic_val: sonic_rs::Value = sonic_rs::from_str(json).unwrap();
            let mut probe_vec = Vec::new();
            sonic_rs::to_writer_pretty(&mut probe_vec, &sonic_val).unwrap();
            let mut sonic_buf: Vec<u8> = Vec::with_capacity(probe_vec.len());
            group.bench_function(BenchmarkId::new("sonic_rs", name), |b| {
                b.iter(|| {
                    sonic_buf.clear();
                    sonic_rs::to_writer_pretty(&mut sonic_buf, black_box(&sonic_val)).unwrap();
                    black_box(&sonic_buf);
                });
            });
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_prettify_ast
}
criterion_main!(benches);
