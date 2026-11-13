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

| method           | jjpwrgem                 | light-json-lsp           | vscode-json                |
| ---------------- | ------------------------ | ------------------------ | -------------------------- |
| initialize       | 861.1µs (3.9 MB)         | 1.1ms (5.4 MB)           | 51.4ms (63.7 MB)           |
| diagnostic       | 46.0µs / 60.1µs (4.6 MB) | 50.2ms / 50.5ms (5.5 MB) | 30.2µs / 39.8µs (137.3 MB) |
| formatting       | 7.7µs / 16.1µs (3.9 MB)  | 18.6µs / 33.4µs (5.5 MB) | 14.9µs / 54.5µs (89.7 MB)  |
| document symbols | 27.5µs / 89.0µs (4.0 MB) | 19.9µs / 89.6µs (5.6 MB) | 17.2µs / 82.1µs (82.2 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                  | light-json-lsp            | vscode-json                 |
| ---------------- | ------------------------- | ------------------------- | --------------------------- |
| initialize       | 861.1µs (3.9 MB)          | 1.1ms (5.4 MB)            | 51.4ms (63.7 MB)            |
| diagnostic       | 1.9ms / 1.9ms (7.8 MB)    | 59.9ms / 61.4ms (16.0 MB) | 7.8ms / 8.1ms (185.8 MB)    |
| formatting       | 705.7µs / 1.3ms (10.4 MB) | 4.1ms / 5.3ms (16.1 MB)   | 13.2ms / 82.9ms (177.2 MB)  |
| document symbols | 14.3ms / 61.0ms (33.0 MB) | 6.9ms / 154.7ms (17.5 MB) | 21.0ms / 186.4ms (199.6 MB) |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                  | light-json-lsp             | vscode-json                 |
| ---------------- | ------------------------- | -------------------------- | --------------------------- |
| initialize       | 861.1µs (3.9 MB)          | 1.1ms (5.4 MB)             | 51.4ms (63.7 MB)            |
| diagnostic       | 2.5ms / 2.6ms (12.1 MB)   | 68.7ms / 71.1ms (23.6 MB)  | 18.3ms / 18.4ms (300.5 MB)  |
| formatting       | 1.1ms / 1.8ms (13.6 MB)   | 8.2ms / 10.6ms (25.4 MB)   | 29.3ms / 213.2ms (313.3 MB) |
| document symbols | 13.4ms / 51.8ms (38.2 MB) | 15.3ms / 359.0ms (30.5 MB) | 52.3ms / 413.9ms (306.8 MB) |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                 | light-json-lsp              | vscode-json                   |
| ---------------- | ------------------------ | --------------------------- | ----------------------------- |
| initialize       | 861.1µs (3.9 MB)         | 1.1ms (5.4 MB)              | 51.4ms (63.7 MB)              |
| diagnostic       | 7.4ms / 7.4ms (25.3 MB)  | 87.7ms / 91.5ms (51.6 MB)   | 38.4ms / 38.6ms (488.8 MB)    |
| formatting       | 2.8ms / 6.7ms (32.0 MB)  | 39.7ms / 49.9ms (64.4 MB)   | 105.3ms / 933.5ms (490.2 MB)  |
| document symbols | 9.8ms / 43.0ms (47.7 MB) | 57.5ms / 1721.2ms (52.7 MB) | 148.4ms / 1763.7ms (419.4 MB) |
