# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

{{BENCH_HARDWARE}}

Diagnostic timing is edit to `publishDiagnostics`. VSCode's JSON LSP waits 500ms after each change before validating. It advertises pull diagnostics but answers `textDocument/diagnostic` with "Unhandled method", so push is the only path

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
