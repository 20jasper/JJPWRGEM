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
| initialize       | 861.1µs (4.0 MB)         | 1.1ms (5.3 MB)           | 50.3ms (63.6 MB)           |
| diagnostic       | 42.1µs / 56.5µs (4.8 MB) | 50.0ms / 50.3ms (5.7 MB) | 29.6µs / 38.9µs (137.1 MB) |
| formatting       | 8.3µs / 17.4µs (4.0 MB)  | 12.3µs / 19.6µs (5.8 MB) | 16.3µs / 57.8µs (83.3 MB)  |
| document symbols | 26.5µs / 89.9µs (4.3 MB) | 12.2µs / 76.1µs (5.6 MB) | 18.2µs / 88.7µs (81.4 MB)  |
| folding ranges   | 8.1µs / 17.5µs (4.2 MB)  | 17.0µs / 30.8µs (5.8 MB) | 9.8µs / 17.7µs (103.1 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                  | light-json-lsp            | vscode-json                 |
| ---------------- | ------------------------- | ------------------------- | --------------------------- |
| initialize       | 861.1µs (4.0 MB)          | 1.1ms (5.3 MB)            | 50.3ms (63.6 MB)            |
| diagnostic       | 1.5ms / 1.5ms (8.6 MB)    | 59.2ms / 61.3ms (16.1 MB) | 5.9ms / 6.3ms (202.2 MB)    |
| formatting       | 677.7µs / 1.3ms (10.5 MB) | 3.1ms / 4.3ms (16.3 MB)   | 9.6ms / 62.1ms (175.6 MB)   |
| document symbols | 14.4ms / 58.4ms (33.5 MB) | 6.8ms / 159.4ms (17.6 MB) | 16.0ms / 170.8ms (190.2 MB) |
| folding ranges   | 959.8µs / 1.7ms (8.5 MB)  | 6.4ms / 8.0ms (15.8 MB)   | 3.7ms / 5.7ms (109.2 MB)    |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                  | light-json-lsp             | vscode-json                 |
| ---------------- | ------------------------- | -------------------------- | --------------------------- |
| initialize       | 861.1µs (4.0 MB)          | 1.1ms (5.3 MB)             | 50.3ms (63.6 MB)            |
| diagnostic       | 2.5ms / 2.6ms (12.3 MB)   | 68.3ms / 70.7ms (23.7 MB)  | 17.7ms / 17.9ms (286.4 MB)  |
| formatting       | 1.0ms / 1.8ms (13.5 MB)   | 9.2ms / 11.5ms (25.3 MB)   | 28.8ms / 206.5ms (310.9 MB) |
| document symbols | 13.4ms / 50.6ms (38.7 MB) | 15.9ms / 366.8ms (30.6 MB) | 51.1ms / 407.6ms (306.5 MB) |
| folding ranges   | 2.0ms / 4.2ms (13.5 MB)   | 27.0ms / 40.2ms (24.8 MB)  | 11.0ms / 19.1ms (167.1 MB)  |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                  | light-json-lsp              | vscode-json                   |
| ---------------- | ------------------------- | --------------------------- | ----------------------------- |
| initialize       | 861.1µs (4.0 MB)          | 1.1ms (5.3 MB)              | 50.3ms (63.6 MB)              |
| diagnostic       | 7.2ms / 7.3ms (25.4 MB)   | 88.1ms / 91.9ms (51.7 MB)   | 38.4ms / 38.6ms (474.4 MB)    |
| formatting       | 3.3ms / 7.2ms (32.1 MB)   | 38.9ms / 48.6ms (64.5 MB)   | 100.0ms / 930.4ms (541.2 MB)  |
| document symbols | 12.0ms / 50.3ms (48.9 MB) | 64.0ms / 1816.3ms (52.9 MB) | 152.0ms / 1831.0ms (316.1 MB) |
| folding ranges   | 2.2ms / 2.6ms (19.7 MB)   | 39.1ms / 40.4ms (49.3 MB)   | 20.3ms / 21.4ms (170.0 MB)    |
