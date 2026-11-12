# LSP benchmarks

Server processing time and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

Cells read `server / total (memory)`: server is time spent in the language server and total includes the client request and responses, which vary based on size of messages. Server time is measured by lspresso's `Client::timings`, from the request being fully written to the first byte of the response. The `initialize` bench is wall clock, from process spawn to the initialize response

Note: vscode sends a large amount of range edit responses, which inflates the time quite a lot

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
