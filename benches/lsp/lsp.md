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

| method           | jjpwrgem                | light-json-lsp           | vscode-json                |
| ---------------- | ----------------------- | ------------------------ | -------------------------- |
| initialize       | 624.6µs (3.5 MB)        | 1.1ms (5.3 MB)           | 48.1ms (62.8 MB)           |
| diagnostic       | 6.4µs / 13.2µs (3.5 MB) | 50.1ms / 50.4ms (5.5 MB) | 15.5µs / 23.0µs (100.6 MB) |
| formatting       | 5.8µs / 13.5µs (3.5 MB) | 9.6µs / 16.6µs (5.5 MB)  | 14.7µs / 53.3µs (82.0 MB)  |
| document symbols | 9.3µs / 69.2µs (3.4 MB) | 12.5µs / 72.7µs (5.5 MB) | 19.8µs / 101.2µs (80.2 MB) |
| folding ranges   | 5.3µs / 12.4µs (3.5 MB) | 13.1µs / 23.0µs (5.6 MB) | 9.6µs / 17.2µs (113.6 MB)  |
| selection ranges | 5.5µs / 20.8µs (3.4 MB) | 11.2µs / 22.8µs (5.6 MB) | 10.9µs / 28.3µs (93.7 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                   | light-json-lsp            | vscode-json                 |
| ---------------- | -------------------------- | ------------------------- | --------------------------- |
| initialize       | 624.6µs (3.5 MB)           | 1.1ms (5.3 MB)            | 48.1ms (62.8 MB)            |
| diagnostic       | 1.5ms / 1.5ms (6.7 MB)     | 60.2ms / 61.5ms (15.9 MB) | 5.9ms / 5.9ms (213.4 MB)    |
| formatting       | 512.5µs / 1.2ms (6.2 MB)   | 3.2ms / 4.2ms (16.0 MB)   | 9.3ms / 63.3ms (175.4 MB)   |
| document symbols | 3.0ms / 42.0ms (8.9 MB)    | 6.6ms / 153.3ms (17.3 MB) | 15.6ms / 171.2ms (159.8 MB) |
| folding ranges   | 875.3µs / 1.7ms (6.1 MB)   | 6.4ms / 8.1ms (15.5 MB)   | 3.0ms / 4.4ms (131.3 MB)    |
| selection ranges | 436.8µs / 472.2µs (5.9 MB) | 23.4µs / 44.7µs (14.3 MB) | 17.2µs / 57.1µs (107.1 MB)  |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                   | light-json-lsp             | vscode-json                 |
| ---------------- | -------------------------- | -------------------------- | --------------------------- |
| initialize       | 624.6µs (3.5 MB)           | 1.1ms (5.3 MB)             | 48.1ms (62.8 MB)            |
| diagnostic       | 2.7ms / 2.8ms (10.3 MB)    | 68.9ms / 71.3ms (23.6 MB)  | 17.8ms / 17.9ms (300.5 MB)  |
| formatting       | 907.8µs / 1.6ms (8.6 MB)   | 7.9ms / 10.5ms (25.3 MB)   | 29.4ms / 212.4ms (287.3 MB) |
| document symbols | 2.9ms / 37.9ms (9.4 MB)    | 15.6ms / 360.8ms (30.3 MB) | 65.9ms / 461.5ms (319.0 MB) |
| folding ranges   | 1.2ms / 3.5ms (7.6 MB)     | 24.7ms / 36.9ms (24.7 MB)  | 11.0ms / 18.9ms (167.6 MB)  |
| selection ranges | 100.8µs / 130.9µs (7.1 MB) | 21.6µs / 47.4µs (22.5 MB)  | 17.7µs / 53.7µs (132.4 MB)  |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                    | light-json-lsp              | vscode-json                   |
| ---------------- | --------------------------- | --------------------------- | ----------------------------- |
| initialize       | 624.6µs (3.5 MB)            | 1.1ms (5.3 MB)              | 48.1ms (62.8 MB)              |
| diagnostic       | 7.4ms / 7.4ms (18.7 MB)     | 90.9ms / 96.0ms (51.5 MB)   | 41.2ms / 41.3ms (471.8 MB)    |
| formatting       | 2.1ms / 5.5ms (21.6 MB)     | 39.4ms / 49.1ms (64.4 MB)   | 99.5ms / 916.4ms (458.2 MB)   |
| document symbols | 2.7ms / 44.6ms (17.7 MB)    | 56.2ms / 1704.9ms (52.6 MB) | 142.8ms / 1760.8ms (364.0 MB) |
| folding ranges   | 2.4ms / 2.7ms (15.5 MB)     | 39.1ms / 40.2ms (49.1 MB)   | 19.1ms / 20.3ms (169.1 MB)    |
| selection ranges | 217.0µs / 271.7µs (15.6 MB) | 19.3µs / 47.2µs (48.7 MB)   | 114.5µs / 152.2µs (166.5 MB)  |
