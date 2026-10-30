use std::hint::black_box;

use criterion::{
    BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main, measurement::WallTime,
};
use jjpwrgem_parse::format::serde::uglify_serializable;
use serde::{Serialize, de::DeserializeOwned};

mod json_common;
mod json_models;

use json_common::{include_impl, load_inputs};
use json_models::{canada::Canada, citm_catalog::CitmCatalog, twitter::Twitter};

fn bench_model<T: Serialize + DeserializeOwned>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    json: &str,
) {
    let model: T = serde_json::from_str(json).unwrap();
    let expected = serde_json::to_string(&model).unwrap();
    assert_eq!(uglify_serializable(&model).unwrap(), expected, "{name}");
    group.throughput(Throughput::Bytes(json.len() as u64));

    if include_impl("jjpwrgem") {
        group.bench_function(format!("jjpwrgem/{name}"), |b| {
            b.iter(|| uglify_serializable(black_box(&model)).unwrap());
        });
    }

    if include_impl("serde_json") {
        group.bench_function(format!("serde_json/{name}"), |b| {
            b.iter(|| serde_json::to_string(black_box(&model)).unwrap());
        });
    }

    if include_impl("simd_json") {
        group.bench_function(format!("simd_json/{name}"), |b| {
            b.iter(|| simd_json::serde::to_string(black_box(&model)).unwrap());
        });
    }

    if include_impl("sonic_rs") {
        group.bench_function(format!("sonic_rs/{name}"), |b| {
            b.iter(|| sonic_rs::to_string(black_box(&model)).unwrap());
        });
    }
}

fn bench_serialize(c: &mut Criterion) {
    let inputs = load_inputs();
    let mut group = c.benchmark_group("serialize_struct");

    for (name, json) in &inputs {
        match *name {
            "canada" => bench_model::<Canada>(&mut group, name, json),
            "citm_catalog" => bench_model::<CitmCatalog>(&mut group, name, json),
            "twitter" => bench_model::<Twitter>(&mut group, name, json),
            _ => unreachable!("no model for {name}"),
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_serialize
}
criterion_main!(benches);
