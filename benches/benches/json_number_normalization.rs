use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jjpwrgem_parse::{
    ast::parse_str,
    format::{LineEnding, prettify_value_into, uglify_value_into},
};

const ZERO_EXPONENT_ARRAY: &str = r#"[
    1e0, 2e00, 3e+00, 4e-00, 5, 6, 7, 8, 9, 10,
    11e0, 12e00, 13e+00, 14e-00, 15, 16, 17, 18, 19, 20,
    21e0, 22e00, 23e+00, 24e-00, 25, 26, 27, 28, 29, 30,
    31e0, 32e00, 33e+00, 34e-00, 35, 36, 37, 38, 39, 40,
    41e0, 42e00, 43e+00, 44e-00, 45, 46, 47, 48, 49, 50
]"#;

const PLAIN_INTEGER_ARRAY: &str = r#"[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50
]"#;

const INPUTS: &[(&str, &str)] = &[
    ("zero_exponents", ZERO_EXPONENT_ARRAY),
    ("plain_integers", PLAIN_INTEGER_ARRAY),
];

fn bench_deser(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_norm_deser");

    for (name, json) in INPUTS {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_function(BenchmarkId::new("jjpwrgem", name), |b| {
            b.iter(|| parse_str(black_box(json)).unwrap());
        });
    }

    group.finish();
}

fn bench_prettify(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_norm_prettify");

    for (name, json) in INPUTS {
        group.throughput(Throughput::Bytes(json.len() as u64));

        let ast = parse_str(json).unwrap();
        let mut probe = String::new();
        prettify_value_into(&mut probe, &ast, 80, LineEnding::Lf);
        let mut buf = String::with_capacity(probe.len());

        group.bench_function(BenchmarkId::new("jjpwrgem", name), |b| {
            b.iter(|| {
                buf.clear();
                prettify_value_into(&mut buf, black_box(&ast), 80, LineEnding::Lf);
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_uglify(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_norm_uglify");

    for (name, json) in INPUTS {
        group.throughput(Throughput::Bytes(json.len() as u64));

        let ast = parse_str(json).unwrap();
        let mut probe = String::new();
        uglify_value_into(&mut probe, &ast);
        let mut buf = String::with_capacity(probe.len());

        group.bench_function(BenchmarkId::new("jjpwrgem", name), |b| {
            b.iter(|| {
                buf.clear();
                uglify_value_into(&mut buf, black_box(&ast));
                black_box(&buf);
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_deser, bench_prettify, bench_uglify
}
criterion_main!(benches);
