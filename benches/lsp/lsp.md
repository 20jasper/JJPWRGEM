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

| method           | jjpwrgem                | light-json-lsp           | vscode-json               |
| ---------------- | ----------------------- | ------------------------ | ------------------------- |
| initialize       | 620.7µs (3.3 MB)        | 1.1ms (5.1 MB)           | 49.6ms (62.7 MB)          |
| diagnostic       | 6.4µs / 13.3µs (3.3 MB) | 50.0ms / 50.4ms (5.5 MB) | 15.4µs / 22.9µs (96.1 MB) |
| formatting       | 5.5µs / 12.7µs (3.5 MB) | 9.4µs / 15.9µs (5.5 MB)  | 15.0µs / 54.1µs (82.2 MB) |
| document symbols | 9.4µs / 72.0µs (3.3 MB) | 11.6µs / 72.8µs (5.4 MB) | 16.1µs / 77.5µs (80.9 MB) |
| folding ranges   | 4.7µs / 11.0µs (3.3 MB) | 13.3µs / 23.1µs (5.6 MB) | 9.6µs / 17.2µs (113.0 MB) |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                 | light-json-lsp            | vscode-json                 |
| ---------------- | ------------------------ | ------------------------- | --------------------------- |
| initialize       | 620.7µs (3.3 MB)         | 1.1ms (5.1 MB)            | 49.6ms (62.7 MB)            |
| diagnostic       | 1.6ms / 1.6ms (6.3 MB)   | 59.8ms / 61.3ms (15.9 MB) | 6.1ms / 6.2ms (185.9 MB)    |
| formatting       | 530.1µs / 1.1ms (6.2 MB) | 4.2ms / 5.3ms (16.1 MB)   | 9.6ms / 61.6ms (172.7 MB)   |
| document symbols | 3.1ms / 43.6ms (8.9 MB)  | 6.8ms / 155.8ms (17.4 MB) | 20.5ms / 190.5ms (171.8 MB) |
| folding ranges   | 775.7µs / 1.4ms (6.0 MB) | 6.5ms / 8.1ms (15.6 MB)   | 3.7ms / 5.7ms (108.1 MB)    |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                 | light-json-lsp             | vscode-json                 |
| ---------------- | ------------------------ | -------------------------- | --------------------------- |
| initialize       | 620.7µs (3.3 MB)         | 1.1ms (5.1 MB)             | 49.6ms (62.7 MB)            |
| diagnostic       | 2.6ms / 2.6ms (10.1 MB)  | 68.6ms / 70.8ms (23.5 MB)  | 17.9ms / 18.0ms (305.1 MB)  |
| formatting       | 989.5µs / 1.9ms (8.4 MB) | 7.9ms / 10.1ms (25.3 MB)   | 35.4ms / 226.4ms (308.9 MB) |
| document symbols | 3.0ms / 36.6ms (9.5 MB)  | 15.2ms / 362.2ms (30.4 MB) | 51.2ms / 408.0ms (318.7 MB) |
| folding ranges   | 1.2ms / 3.7ms (7.6 MB)   | 25.5ms / 39.8ms (24.6 MB)  | 12.1ms / 23.5ms (165.2 MB)  |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                 | light-json-lsp              | vscode-json                   |
| ---------------- | ------------------------ | --------------------------- | ----------------------------- |
| initialize       | 620.7µs (3.3 MB)         | 1.1ms (5.1 MB)              | 49.6ms (62.7 MB)              |
| diagnostic       | 7.3ms / 7.4ms (18.6 MB)  | 88.6ms / 92.5ms (51.5 MB)   | 40.2ms / 40.3ms (497.0 MB)    |
| formatting       | 2.5ms / 6.1ms (21.4 MB)  | 38.8ms / 48.3ms (64.5 MB)   | 99.0ms / 885.5ms (535.8 MB)   |
| document symbols | 1.9ms / 37.2ms (17.6 MB) | 57.3ms / 1704.3ms (52.6 MB) | 149.9ms / 1780.2ms (430.7 MB) |
| folding ranges   | 2.3ms / 2.8ms (15.5 MB)  | 37.9ms / 38.8ms (49.2 MB)   | 19.2ms / 20.3ms (169.0 MB)    |
