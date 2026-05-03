use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jjpwrgem_parse::tokens::TokenStream;

mod json_common;

use json_common::{include_impl, load_inputs};

fn bench_tokens(c: &mut Criterion) {
    let inputs = load_inputs();
    let mut group = c.benchmark_group("tokens");

    for (name, json) in &inputs {
        group.throughput(Throughput::Bytes(json.len() as u64));

        if include_impl("jjpwrgem") {
            group.bench_with_input(
                BenchmarkId::new("jjpwrgem", name),
                json.as_str(),
                |b, json| {
                    b.iter(|| {
                        TokenStream::new(black_box(json)).for_each(|token| {
                            black_box(token.unwrap());
                        });
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_tokens
}
criterion_main!(benches);
