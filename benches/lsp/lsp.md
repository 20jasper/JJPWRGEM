<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP and light-json-lsp, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp  | vscode-json       |
| ---------- | ---------------- | --------------- | ----------------- |
| initialize | 858.1µs (4.1 MB) | 1.1ms (5.4 MB)  | 78.1ms (72.3 MB)  |
| diagnostic | 65.0µs (4.6 MB)  | 50.1ms (5.6 MB) | 44.5µs (184.5 MB) |
| formatting | 21.6µs (4.1 MB)  | 17.0µs (5.6 MB) | 54.9µs (107.8 MB) |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json       |
| ---------- | ---------------- | ---------------- | ----------------- |
| initialize | 858.1µs (4.1 MB) | 1.1ms (5.4 MB)   | 78.1ms (72.3 MB)  |
| diagnostic | 3.7ms (7.8 MB)   | 58.0ms (15.9 MB) | 6.9ms (190.9 MB)  |
| formatting | 1.3ms (10.5 MB)  | 5.4ms (16.1 MB)  | 68.6ms (180.4 MB) |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json        |
| ---------- | ---------------- | ---------------- | ------------------ |
| initialize | 858.1µs (4.1 MB) | 1.1ms (5.4 MB)   | 78.1ms (72.3 MB)   |
| diagnostic | 4.0ms (11.8 MB)  | 65.9ms (23.5 MB) | 20.5ms (294.9 MB)  |
| formatting | 1.8ms (13.3 MB)  | 11.8ms (25.2 MB) | 233.0ms (279.8 MB) |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json         |
| ---------- | ---------------- | ---------------- | ------------------- |
| initialize | 858.1µs (4.1 MB) | 1.1ms (5.4 MB)   | 78.1ms (72.3 MB)    |
| diagnostic | 10.9ms (22.4 MB) | 92.6ms (51.6 MB) | 42.2ms (486.1 MB)   |
| formatting | 6.7ms (32.1 MB)  | 48.7ms (64.4 MB) | 1058.5ms (464.1 MB) |
