# jjpwrgem benchmarks

{{BENCH_SUMMARY}}

- [CLI formatter and minifier speed](cli-formatter.md): wall-clock timing against jq, prettier, dprint, and others
- [JSON syntax tree serialization and deserialization throughput](json.md): throughput and wall clock benchmarks vs serde_json, simd_json, and sonic-rs
- [lazy UTF-8 decode throughput](utf8.md): `bytes2chars` vs `utf8-decode` and `bstr`
