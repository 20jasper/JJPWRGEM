# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP via `lsp-bench`. Run locally with `mise run bench-lsp-all`

{{BENCH_HARDWARE}}

VSCode's diagnostics calculation has a 500ms delay for push only diagnostics. The LSP client can pull as of LSP 3.17 (2022), so most modern editors request diagnostics earlier than 500ms

JJPWRGEM does not delay anything

## small

{{FIXTURE_SMALL}}

Baseline performance for minimal parsing work

{{LSP_BENCH_SMALL_TABLE}}

## twitter

{{FIXTURE_TWITTER}}

{{LSP_BENCH_TWITTER_TABLE}}

## citm catalog

{{FIXTURE_CITM_CATALOG}}

{{LSP_BENCH_CITM_CATALOG_TABLE}}

## canada

{{FIXTURE_CANADA}}

{{LSP_BENCH_CANADA_TABLE}}
