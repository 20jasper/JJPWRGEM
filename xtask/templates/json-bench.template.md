# jjpwrgem JSON benchmarks

{{BENCH_SUMMARY}}

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `mise run bench-json` or individual `mise run bench json_deser`, `mise run bench json_prettify`, and `mise run bench json_uglify`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

{{BENCH_INPUTS}}

{{JSON_DESER_BENCH_TABLE}}

{{JSON_PRETTIFY_BENCH_TABLE}}

{{JSON_UGLIFY_BENCH_TABLE}}
