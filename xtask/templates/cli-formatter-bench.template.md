# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `just bench-docker`
{{BENCH_HARDWARE}}

{{BENCH_INPUTS}}

## canada

{{FIXTURE_CANADA}}

### pretty canada

![candlestick benchmark for pretty printing canada.json](/xtask/bench/output/pretty-canada.png)

{{PRETTY_CANADA_TABLE}}

### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

{{UGLY_CANADA_TABLE}}

## citm catalog

{{FIXTURE_CITM_CATALOG}}

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

{{PRETTY_CITM_CATALOG_TABLE}}

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

{{UGLY_CITM_CATALOG_TABLE}}

## twitter

{{FIXTURE_TWITTER}}

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

{{PRETTY_TWITTER_TABLE}}

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

{{UGLY_TWITTER_TABLE}}
