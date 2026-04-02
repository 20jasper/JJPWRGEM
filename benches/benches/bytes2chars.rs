use std::hint::black_box;

use bstr::ByteSlice as _;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn ascii_input(min_size: usize) -> Vec<u8> {
    let unit = b"Hello, World! This is ASCII benchmark input for UTF-8 decoding. ";
    let reps = (min_size / unit.len()) + 1;
    unit.repeat(reps)
}

fn unicode_input(min_size: usize) -> Vec<u8> {
    let unit = "🦀 résumé 日本語 Ünïcödé ";
    let reps = (min_size / unit.len()) + 1;
    unit.repeat(reps).into_bytes()
}

fn bench_group(c: &mut Criterion, name: &str, inputs: &[(usize, Vec<u8>)]) {
    let mut group = c.benchmark_group(name);

    for (size, bytes) in inputs {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("bytes2chars", size), bytes, |b, bytes| {
            b.iter(|| {
                bytes2chars::Utf8Chars::from(black_box(bytes).iter().copied()).for_each(|r| {
                    let _ = black_box(r.unwrap());
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("utf8_decode", size), bytes, |b, bytes| {
            b.iter(|| {
                utf8_decode::Decoder::new(black_box(bytes).iter().copied()).for_each(
                    |r: Result<char, _>| {
                        let _ = black_box(r.unwrap());
                    },
                );
            });
        });

        group.bench_with_input(BenchmarkId::new("bstr", size), bytes, |b, bytes| {
            b.iter(|| {
                black_box(bytes).chars().for_each(|c| {
                    let _ = black_box(c);
                });
            });
        });
    }

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    let sizes = [64 * 1024];
    let ascii: Vec<(usize, Vec<u8>)> = sizes.map(|s| (s, ascii_input(s))).into();
    let unicode: Vec<(usize, Vec<u8>)> = sizes.map(|s| (s, unicode_input(s))).into();
    bench_group(c, "ascii", &ascii);
    bench_group(c, "non_ascii", &unicode);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
