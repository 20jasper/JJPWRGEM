<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Diagnostic timing is edit to diagnostics availability: an immediate `textDocument/diagnostic` pull where the server supports it, otherwise its pushed `publishDiagnostics`

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp  | vscode-json       |
| ---------- | ---------------- | --------------- | ----------------- |
| initialize | 832.3µs (4.2 MB) | 1.2ms (5.1 MB)  | 50.0ms (63.5 MB)  |
| diagnostic | 19.2µs (4.1 MB)  | 50.2ms (5.5 MB) | 26.1µs (117.9 MB) |
| formatting | 17.5µs (4.1 MB)  | 16.1µs (5.4 MB) | 56.5µs (90.4 MB)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json       |
| ---------- | ---------------- | ---------------- | ----------------- |
| initialize | 832.3µs (4.2 MB) | 1.2ms (5.1 MB)   | 50.0ms (63.5 MB)  |
| diagnostic | 2.0ms (9.1 MB)   | 57.1ms (25.9 MB) | 8.1ms (194.9 MB)  |
| formatting | 1.2ms (10.6 MB)  | 4.4ms (16.0 MB)  | 71.6ms (177.5 MB) |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp   | vscode-json        |
| ---------- | ---------------- | ---------------- | ------------------ |
| initialize | 832.3µs (4.2 MB) | 1.2ms (5.1 MB)   | 50.0ms (63.5 MB)   |
| diagnostic | 4.2ms (14.0 MB)  | 64.6ms (44.5 MB) | 22.1ms (276.4 MB)  |
| formatting | 1.8ms (13.4 MB)  | 10.6ms (25.3 MB) | 261.6ms (311.6 MB) |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method     | jjpwrgem         | light-json-lsp     | vscode-json         |
| ---------- | ---------------- | ------------------ | ------------------- |
| initialize | 832.3µs (4.2 MB) | 1.2ms (5.1 MB)     | 50.0ms (63.5 MB)    |
| diagnostic | 11.0ms (32.0 MB) | 135.4ms (104.6 MB) | 48.2ms (518.1 MB)   |
| formatting | 7.0ms (32.1 MB)  | 48.3ms (64.4 MB)   | 1070.4ms (525.1 MB) |
