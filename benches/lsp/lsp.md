<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Server processing time and memory usage against VSCode's JSON LSP and light-json-lsp, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

Cells read `server / total (memory)`: server is time spent in the language server and total includes the client request and responses, which vary based on size of messages. Server time is measured by lspresso's `Client::timings`, from the request being fully written to the first byte of the response. The `initialize` bench is wall clock, from process spawn to the initialize response

Note: vscode sends a large amount of range edit responses, which inflates the time quite a lot

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem                 | light-json-lsp           | vscode-json                |
| ---------- | ------------------------ | ------------------------ | -------------------------- |
| initialize | 844.1µs (4.0 MB)         | 1.2ms (5.3 MB)           | 51.1ms (63.7 MB)           |
| diagnostic | 42.8µs / 57.6µs (5.0 MB) | 49.8ms / 50.4ms (5.5 MB) | 30.3µs / 39.6µs (137.8 MB) |
| formatting | 8.3µs / 17.3µs (4.2 MB)  | 9.2µs / 15.5µs (5.4 MB)  | 15.5µs / 56.5µs (84.4 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem                  | light-json-lsp            | vscode-json               |
| ---------- | ------------------------- | ------------------------- | ------------------------- |
| initialize | 844.1µs (4.0 MB)          | 1.2ms (5.3 MB)            | 51.1ms (63.7 MB)          |
| diagnostic | 1.5ms / 1.5ms (8.4 MB)    | 59.8ms / 61.1ms (15.8 MB) | 5.8ms / 5.9ms (175.5 MB)  |
| formatting | 645.8µs / 1.2ms (10.6 MB) | 3.9ms / 5.1ms (16.0 MB)   | 9.7ms / 64.4ms (177.3 MB) |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem                | light-json-lsp            | vscode-json                 |
| ---------- | ----------------------- | ------------------------- | --------------------------- |
| initialize | 844.1µs (4.0 MB)        | 1.2ms (5.3 MB)            | 51.1ms (63.7 MB)            |
| diagnostic | 2.5ms / 2.6ms (12.1 MB) | 68.7ms / 70.8ms (23.6 MB) | 18.0ms / 18.1ms (310.1 MB)  |
| formatting | 1.1ms / 1.8ms (13.6 MB) | 9.3ms / 11.6ms (25.4 MB)  | 29.3ms / 214.9ms (302.5 MB) |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem                | light-json-lsp            | vscode-json                  |
| ---------- | ----------------------- | ------------------------- | ---------------------------- |
| initialize | 844.1µs (4.0 MB)        | 1.2ms (5.3 MB)            | 51.1ms (63.7 MB)             |
| diagnostic | 7.5ms / 7.6ms (25.4 MB) | 91.2ms / 96.5ms (51.5 MB) | 40.9ms / 41.0ms (476.3 MB)   |
| formatting | 2.7ms / 6.5ms (32.1 MB) | 39.2ms / 49.0ms (64.4 MB) | 102.9ms / 950.4ms (495.0 MB) |
