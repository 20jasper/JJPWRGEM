# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `mise run bench-docker`
{% include "bench_hardware.md" %}

{% include "bench_inputs.md" %}

## canada

{% include "fixture_canada.md" %}

### pretty canada

![candlestick benchmark for pretty printing canada.json](/benches/docker/output/pretty-canada.svg)

{% include "benches/docker/output/pretty-canada.md" %}

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.svg)

{% include "benches/docker/output/ugly-canada.md" %}

## citm catalog

{% include "fixture_citm_catalog.md" %}

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.svg)

{% include "benches/docker/output/pretty-citm_catalog.md" %}

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.svg)

{% include "benches/docker/output/ugly-citm_catalog.md" %}

## twitter

{% include "fixture_twitter.md" %}

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.svg)

{% include "benches/docker/output/pretty-twitter.md" %}

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.svg)

{% include "benches/docker/output/ugly-twitter.md" %}
