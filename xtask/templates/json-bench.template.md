# jjpwrgem JSON benchmarks

{% include "bench_summary.md" %}

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `mise run bench-json` or individual `mise run bench json_deser`, `mise run bench json_prettify`, `mise run bench json_uglify`, and `mise run bench json_serialize`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

{% include "bench_inputs.md" %}

{% include "benches/output/json_deser.md" %}

{% include "benches/output/json_prettify.md" %}

{% include "benches/output/json_uglify.md" %}

{% include "benches/output/json_serialize.md" %}
