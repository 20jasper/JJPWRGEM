# LSP benchmarks

Server processing time and memory usage against VSCode's JSON LSP and light-json-lsp, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

Cells read `server / total (memory)`: server is time spent in the language server and total includes the client request and responses, which vary based on size of messages. Server time is measured by lspresso's `Client::timings`, from the request being fully written to the first byte of the response. The `initialize` bench is wall clock, from process spawn to the initialize response

Note: vscode sends a large amount of range edit responses, which inflates the time quite a lot

{% include "bench_hardware.md" %}

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

## small

{% include "fixture_small.md" %}

Baseline performance for minimal parsing work

{% include "benches/lsp/results/small.md" %}

## twitter

{% include "fixture_twitter.md" %}

{% include "benches/lsp/results/twitter.md" %}

## citm catalog

{% include "fixture_citm_catalog.md" %}

{% include "benches/lsp/results/citm_catalog.md" %}

## canada

{% include "fixture_canada.md" %}

{% include "benches/lsp/results/canada.md" %}
