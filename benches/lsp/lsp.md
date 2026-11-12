<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Server processing time and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

Timings are the server's processing time as measured by lspresso's `Client::timings`: from the request being fully written to the first byte of the response, excluding pipe IO and client serialization and parsing. The `initialize` bench is wall clock, from process spawn to the initialize response

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp  | vscode-json       |
| ---------- | ---------------- | --------------- | ----------------- |
| initialize | 856.1µs (4.0 MB) | 1.2ms (5.3 MB)  | 49.0ms (63.5 MB)  |
| diagnostic | 42.7µs (5.0 MB)  | 50.0ms (5.6 MB) | 31.0µs (140.9 MB) |
| formatting | 8.2µs (4.2 MB)   | 9.5µs (5.6 MB)  | 16.2µs (83.1 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem          | light-json-lsp   | vscode-json      |
| ---------- | ----------------- | ---------------- | ---------------- |
| initialize | 856.1µs (4.0 MB)  | 1.2ms (5.3 MB)   | 49.0ms (63.5 MB) |
| diagnostic | 1.7ms (8.5 MB)    | 59.6ms (15.9 MB) | 5.7ms (214.0 MB) |
| formatting | 595.8µs (10.7 MB) | 3.4ms (16.1 MB)  | 9.3ms (176.4 MB) |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json       |
| ---------- | ---------------- | ---------------- | ----------------- |
| initialize | 856.1µs (4.0 MB) | 1.2ms (5.3 MB)   | 49.0ms (63.5 MB)  |
| diagnostic | 3.1ms (12.3 MB)  | 68.5ms (23.6 MB) | 18.8ms (302.6 MB) |
| formatting | 1.1ms (13.5 MB)  | 8.4ms (25.3 MB)  | 30.0ms (311.6 MB) |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json        |
| ---------- | ---------------- | ---------------- | ------------------ |
| initialize | 856.1µs (4.0 MB) | 1.2ms (5.3 MB)   | 49.0ms (63.5 MB)   |
| diagnostic | 7.6ms (25.5 MB)  | 91.7ms (51.6 MB) | 40.8ms (476.9 MB)  |
| formatting | 2.7ms (32.0 MB)  | 39.5ms (64.5 MB) | 108.2ms (559.3 MB) |
