# jjpwrgem JSON benchmarks

Throughput benchmarks for `jjpwrgem`'s parse and formatting operations.
Run locally with `just bench-json` or individual `just bench json_deser`, `just bench json_prettify`, and `just bench json_uglify`.
Throughput is normalized by input bytes for all tables so implementations stay comparable even when output formatting differs.

{{JSON_DESER_BENCH_TABLE}}

{{JSON_PRETTIFY_BENCH_TABLE}}

{{JSON_UGLIFY_BENCH_TABLE}}
