# LSP benchmarks

Server processing time and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

Timings are the server's processing time as measured by lspresso's `Client::timings`: from the request being fully written to the first byte of the response, excluding pipe IO and client serialization and parsing. The `initialize` bench is wall clock, from process spawn to the initialize response

{{BENCH_HARDWARE}}

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

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
