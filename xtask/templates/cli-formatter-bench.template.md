# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `mise run bench-docker`
{{BENCH_HARDWARE}}

{{BENCH_INPUTS}}

## canada

{{FIXTURE_CANADA}}

### pretty canada

![candlestick benchmark for pretty printing canada.json](/benches/docker/output/pretty-canada.png)

{{PRETTY_CANADA_TABLE}}

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

{{UGLY_CANADA_TABLE}}

## citm catalog

{{FIXTURE_CITM_CATALOG}}

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

{{PRETTY_CITM_CATALOG_TABLE}}

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

{{UGLY_CITM_CATALOG_TABLE}}

## twitter

{{FIXTURE_TWITTER}}

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

{{PRETTY_TWITTER_TABLE}}

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

{{UGLY_TWITTER_TABLE}}
