<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP, driven by [lspresso](/crates/lspresso) with VSCode's client capabilities. Run locally with `mise run bench-lsp`

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Diagnostic timing is edit to `publishDiagnostics`. VSCode's JSON LSP waits 500ms after each change before validating. It advertises pull diagnostics but answers `textDocument/diagnostic` with "Unhandled method", so push is the only path

JJPWRGEM does not delay anything

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem       | light-json-lsp  | vscode-json       |
| ---------------- | -------------- | --------------- | ----------------- |
| initialize       | 1.1ms (3.3 MB) | 1.6ms (5.3 MB)  | 127.3ms (72.7 MB) |
| diagnostic       | 0.2ms (3.7 MB) | 52.0ms (5.7 MB) | 514.3ms (73.0 MB) |
| formatting       | 0.2ms (3.6 MB) | 0.1ms (5.7 MB)  | 0.2ms (90.0 MB)   |
| formatting edits | 1 edit         | 1 edit          | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | light-json-lsp   | vscode-json        |
| ---------------- | --------------- | ---------------- | ------------------ |
| initialize       | 1.1ms (3.3 MB)  | 1.6ms (5.3 MB)   | 127.3ms (72.7 MB)  |
| diagnostic       | 3.4ms (9.0 MB)  | 62.4ms (25.8 MB) | 540.7ms (148.5 MB) |
| formatting       | 2.3ms (10.1 MB) | 6.9ms (16.3 MB)  | 93.9ms (184.5 MB)  |
| formatting edits | 1 edit          | 1 edit           | 15481 edits        |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | light-json-lsp   | vscode-json        |
| ---------------- | --------------- | ---------------- | ------------------ |
| initialize       | 1.1ms (3.3 MB)  | 1.6ms (5.3 MB)   | 127.3ms (72.7 MB)  |
| diagnostic       | 6.5ms (13.6 MB) | 73.5ms (42.6 MB) | 540.8ms (276.1 MB) |
| formatting       | 3.3ms (12.8 MB) | 16.1ms (25.4 MB) | 368.6ms (326.7 MB) |
| formatting edits | 1 edit          | 1 edit           | 50468 edits        |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | light-json-lsp     | vscode-json         |
| ---------------- | ---------------- | ------------------ | ------------------- |
| initialize       | 1.1ms (3.3 MB)   | 1.6ms (5.3 MB)     | 127.3ms (72.7 MB)   |
| diagnostic       | 18.4ms (28.6 MB) | 168.9ms (107.3 MB) | 580.9ms (365.1 MB)  |
| formatting       | 8.1ms (31.6 MB)  | 62.9ms (64.5 MB)   | 1104.1ms (563.2 MB) |
| formatting edits | 1 edit           | 1 edit             | 223227 edits        |
