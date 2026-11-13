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
| initialize       | 853.0µs (4.1 MB)         | 1.1ms (5.4 MB)           | 82.5ms (72.5 MB)           |
| diagnostic       | 45.5µs / 60.1µs (4.7 MB) | 50.0ms / 50.2ms (5.7 MB) | 33.4µs / 42.5µs (182.6 MB) |
| formatting       | 8.3µs / 17.4µs (4.1 MB)  | 9.8µs / 16.9µs (5.8 MB)  | 15.7µs / 55.8µs (106.8 MB) |
| document symbols | 27.0µs / 89.9µs (4.2 MB) | 18.6µs / 87.9µs (5.7 MB) | 20.4µs / 94.8µs (98.5 MB)  |
| folding ranges   | 8.4µs / 18.2µs (4.2 MB)  | 12.9µs / 22.9µs (5.7 MB) | 10.4µs / 18.7µs (117.3 MB) |
| selection ranges | 15.4µs / 37.2µs (4.2 MB) | 10.2µs / 22.0µs (5.8 MB) | 11.5µs / 29.1µs (116.0 MB) |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                   | light-json-lsp            | vscode-json                 |
| ---------------- | -------------------------- | ------------------------- | --------------------------- |
| initialize       | 853.0µs (4.1 MB)           | 1.1ms (5.4 MB)            | 82.5ms (72.5 MB)            |
| diagnostic       | 1.5ms / 1.6ms (8.5 MB)     | 59.5ms / 61.2ms (16.2 MB) | 6.2ms / 6.3ms (188.3 MB)    |
| formatting       | 669.0µs / 1.3ms (10.5 MB)  | 3.6ms / 5.1ms (16.2 MB)   | 8.9ms / 60.3ms (181.1 MB)   |
| document symbols | 15.5ms / 62.5ms (33.6 MB)  | 8.2ms / 174.7ms (17.5 MB) | 16.9ms / 172.8ms (200.2 MB) |
| folding ranges   | 980.2µs / 1.6ms (8.3 MB)   | 6.6ms / 8.1ms (15.7 MB)   | 3.0ms / 4.6ms (111.6 MB)    |
| selection ranges | 448.4µs / 483.4µs (7.7 MB) | 19.6µs / 41.9µs (14.6 MB) | 17.2µs / 55.5µs (117.6 MB)  |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                    | light-json-lsp             | vscode-json                 |
| ---------------- | --------------------------- | -------------------------- | --------------------------- |
| initialize       | 853.0µs (4.1 MB)            | 1.1ms (5.4 MB)             | 82.5ms (72.5 MB)            |
| diagnostic       | 2.5ms / 2.5ms (12.1 MB)     | 68.6ms / 70.8ms (23.7 MB)  | 18.2ms / 18.3ms (294.7 MB)  |
| formatting       | 1.1ms / 1.8ms (13.5 MB)     | 8.3ms / 10.7ms (25.6 MB)   | 26.2ms / 206.8ms (280.3 MB) |
| document symbols | 13.1ms / 49.8ms (38.6 MB)   | 15.0ms / 360.9ms (30.5 MB) | 47.1ms / 413.2ms (297.0 MB) |
| folding ranges   | 2.2ms / 4.8ms (13.4 MB)     | 26.1ms / 37.8ms (24.8 MB)  | 11.1ms / 19.4ms (156.4 MB)  |
| selection ranges | 111.6µs / 142.1µs (10.1 MB) | 17.3µs / 37.2µs (22.7 MB)  | 17.5µs / 50.8µs (124.3 MB)  |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem                    | light-json-lsp              | vscode-json                   |
| ---------------- | --------------------------- | --------------------------- | ----------------------------- |
| initialize       | 853.0µs (4.1 MB)            | 1.1ms (5.4 MB)              | 82.5ms (72.5 MB)              |
| diagnostic       | 8.4ms / 8.6ms (25.5 MB)     | 87.5ms / 91.7ms (51.8 MB)   | 39.3ms / 39.5ms (488.5 MB)    |
| formatting       | 3.3ms / 7.6ms (32.0 MB)     | 40.5ms / 50.7ms (64.5 MB)   | 110.7ms / 934.6ms (530.2 MB)  |
| document symbols | 11.0ms / 45.8ms (48.7 MB)   | 58.9ms / 1750.6ms (52.7 MB) | 186.5ms / 1895.5ms (313.3 MB) |
| folding ranges   | 2.4ms / 2.8ms (19.6 MB)     | 39.4ms / 40.6ms (49.3 MB)   | 17.6ms / 18.2ms (185.4 MB)    |
| selection ranges | 233.3µs / 273.7µs (20.4 MB) | 42.4µs / 75.5µs (48.8 MB)   | 103.3µs / 142.4µs (169.7 MB)  |
